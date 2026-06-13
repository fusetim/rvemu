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

pub mod shared_mem;

const EXECUTION_HALT : i32 = 0;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    #[link_name = "keepalive"]
    unsafe fn keepalive(i: i32);
    #[link_name = "mem_debug"]
    unsafe fn __mem_debug(tag: i32);
}

#[inline(always)]
pub fn mem_debug(tag: i32) {
    // SAFETY: This is a debug function provided by the host environment.
    //         Any tag / value is okay to pass.
    unsafe { __mem_debug(tag) }
}


#[unsafe(export_name = "run")]
pub fn run() {
    let timeout_ns: i64 = 1_000_000_000; // 1 second in nanoseconds
    let mut i = 42i32;

    // DEBUG: Use the shared memory
    shared_mem::store_word(0, 42);
    assert!(shared_mem::load_word(0) == 42, "Shared memory load/store failed");

    let buf = [0, 1, 2, 3, 4, 5, 6, 7];
    mem_debug(0);
    let copied = shared_mem::copyto(0, &buf);
    assert!(copied == buf.len(), "Shared memory copyto failed");
    mem_debug(1);
    let mut read_buf = [0; 8];
    let copied_back = shared_mem::copyfrom(0, &mut read_buf);
    mem_debug(2);
    assert!(copied_back == buf.len(), "Shared memory copyfrom failed");
    assert!(read_buf == buf, "Shared memory copy mismatch");

    loop {
        unsafe {
            keepalive(i);
        }
        shared_mem::atomic_store(EXECUTION_HALT, i);
        let wait_result = shared_mem::atomic_wait(EXECUTION_HALT, i, timeout_ns);
        if wait_result == 0 {
            // Woken up by a store to EXECUTION_HALT, time to halt execution
            break;
        }
        i+=1;
    }
}