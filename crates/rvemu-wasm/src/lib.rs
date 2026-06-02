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
}

#[unsafe(export_name = "run")]
pub fn run() {
    let timeout_ns: i64 = 1_000_000_000; // 1 second in nanoseconds
    let mut i = 42i32;

    // DEBUG: Use the shared memory
    shared_mem::store_word(0, 42);
    assert!(shared_mem::load_word(0) == 42, "Shared memory load/store failed");

    let buf = [0, 1, 2, 3, 4, 5, 6, 7];
     for (i, &b) in buf.iter().enumerate() {
        shared_mem::debug(i as i32);
        shared_mem::debug(b as i32);
    }
    let copied = shared_mem::copyto(0, &buf);
    assert!(copied == buf.len(), "Shared memory copyto failed");
    let mut read_buf = [0; 8];
    let copied_back = shared_mem::copyfrom(0, &mut read_buf);
    shared_mem::debug(1101);
    for (i, &b) in read_buf.iter().enumerate() {
        shared_mem::debug(i as i32);
        shared_mem::debug(b as i32);
    }
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