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

#[unsafe(export_name = "FOO")]
pub static mut FOO: i32 = 42;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    #[link_name = "keepalive"]
    unsafe fn keepalive(i: i32);
}

#[unsafe(export_name = "run")]
pub fn run() {
    let timeout_ns: i64 = 1_000_000_000; // 1 second in nanoseconds
    let mut i = 42i32;

    // DEBUG: Use the shared memory
    shared_mem::store_word(0, 42);
    assert!(shared_mem::load_word(0) == 42, "Shared memory load/store failed");

    //let buf = [0, 1, 2, 3, 4, 5, 6, 7];
    //let copied = shared_mem::copyto(0, &buf);
    //assert!(copied == buf.len(), "Shared memory copyto failed");
    //let mut read_buf = [0; 8];
    //let copied_back = shared_mem::copyfrom(0, &mut read_buf);
    //assert!(copied_back == buf.len(), "Shared memory copyfrom failed");
    //assert!(read_buf == buf, "Shared memory copy mismatch");

    loop {
        unsafe {
            keepalive(i);
        }
        atomic_wait(i, timeout_ns);
        i+=1;
    }
}

#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "atomics")]
fn atomic_wait(i: i32, timeout_ns: i64) {
    unsafe {
        core::intrinsics::atomic_store::<i32, { AtomicOrdering::Relaxed }>(&raw mut FOO, i);
        core::arch::wasm32::memory_atomic_wait32(&raw mut FOO, i, timeout_ns);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn atomic_wait(i: i32, timeout_ns: i64) {
    // Fallback implementation for non-WASM targets
}