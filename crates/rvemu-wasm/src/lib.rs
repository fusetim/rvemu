#![no_std]
#![feature(abort_immediate)]
#![feature(stdarch_wasm_atomic_wait)]
#![feature(wasm_target_feature)]
#![feature(core_intrinsics)]
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod nostd;

#[cfg(target_arch = "wasm32")]
use core::{intrinsics::AtomicOrdering};

use alloc::{format, vec, vec::Vec};
use rvemu::{executor::rv32i::ExecutorRV32I, reg::Regs32};
use rvemu::executor::Executor;

use crate::{memory::SimpleMemoryController, utils::{HostMemoryHandle, console_log, extract_word}};

pub mod shared_mem;
pub mod utils;
pub mod memory;

#[link(wasm_import_module = "rvemu_host")]
unsafe extern "C" {
    /// Load the ROM from the host environment into the wasm module's main memory.
    /// 
    /// # Safety
    /// This function is unsafe because it involves raw pointers and memory manipulation.
    /// Importantly, you must free the handle obtained, otherwise it will leave forever in host memory. 
    ///
    /// # Returns
    /// 
    /// - `usize`: An handle to a HostMemoryHandle object in the host environment's memory.
    /// 
    #[link_name = "load_rom"]
    unsafe fn rvemu_host__load_rom() -> usize;

    /// Hint visible registers state to the host environment
    /// 
    /// This function call allows the environment to be aware that the current state 
    /// of the emulator (PC, registers) is now visible / copied to the shared memory region, 
    /// and that the host environment can now read it.
    /// 
    /// Once the function returns, the states will be copied back to the internal emulator state.
    /// So it allows the host environment to write too to the shared memory region, and the emulator will read it back.
    /// 
    /// # Safety
    /// 
    /// Quite safe, but exceptions etc are not handled, so if the host environment does something wrong, it can cause issues.
    #[link_name = "hint_visible_registers_state"]
    unsafe fn rvemu_host__hint_visible_registers_state();
}

/// Load the ROM from the host environment into the wasm module's main memory.
pub fn host_load_rom() -> Result<Vec<u8>, ()> {
    let handle = unsafe { rvemu_host__load_rom() };
    console_log(format!("load_rom from indirect handle: {}", handle));
    let handle = unsafe { HostMemoryHandle::from_indirect(handle) };
    console_log(format!("load_rom from handle: {}, len: {}", handle.get_handle(), handle.get_len()));
    if handle.is_null() { 
        return Err(()); 
    }
    let mut rom = vec![0u8; handle.get_len()];
    let copied_len = unsafe { handle.copy_from_host(&mut rom ) };
    console_log(format!("Copied {} bytes from host memory", copied_len));
    if copied_len != handle.get_len() {
        // Free the handle in the host environment to avoid memory leaks
        unsafe { handle.free() };
        return Err(());
    }
    unsafe { handle.free() };
    Ok(rom)
}

pub fn host_hint_visible_registers_state() {
    unsafe { rvemu_host__hint_visible_registers_state() };
}

const SHARED_STATUS_BASE_ADDR : i32 = 0x0000;

const EXECUTION_CONTROL_BASE_ADDR : usize = 0x1000;
const EXECUTION_HALT : usize = EXECUTION_CONTROL_BASE_ADDR + 0;

#[unsafe(export_name = "run")]
pub fn run() {
    // Load the initial environment state from the host side
    let Ok(rom) = host_load_rom() else {
        console_log("Failed to load rom from host environment");
        return;
    };
    console_log(format!("Loaded rom (length: {} bytes)", rom.len()));
    let mut memory = SimpleMemoryController::from_vec(rom);

    // Load the initial state of the registers from the shared memory region
    let mut regs : Regs32 = Regs32::new();
    host_hint_visible_registers_state();
    {
        let mut buf : [u8; 4 * 33] = [0u8; 4 * 33]; // 32 registers + PC
        shared_mem::copyfrom(SHARED_STATUS_BASE_ADDR, &mut buf);
        for i in 0..32 {
            let reg_value : u32 = extract_word!(buf, i);
            regs.write(i, reg_value);
        }
        let pc_value : u32 = extract_word!(buf, 32);
        regs.write_pc(pc_value);
    }
    console_log(format!("Loaded PC: {}", regs.read_pc()));

    let mut executor = ExecutorRV32I::new(memory, regs);

    loop {
        if let Err(e) = executor.step() {
            console_log(format!("Execution error at PC=0x{:08x}: {:?}", executor.read_pc(), e));
            break;
        }
    }

    // Save the final state of the registers to the shared memory region
    {
        let mut buf : [u8; 4 * 33] = [0u8; 4 * 33]; // 32 registers + PC
        for i in 0..32 {
            let reg_value : u32 = executor.regs().read(i);
            buf[i*4..(i+1)*4].copy_from_slice(&reg_value.to_le_bytes());
        }
        let pc_value : u32 = executor.read_pc();
        buf[32*4..33*4].copy_from_slice(&pc_value.to_le_bytes());
        shared_mem::copyto(SHARED_STATUS_BASE_ADDR, &buf);
    }
    host_hint_visible_registers_state();

    // Just to debug, write the registers
    for i in 0..32 {
        console_log(format!("r{} -> {:08x}", i, executor.regs().read(i)));
    }
}
