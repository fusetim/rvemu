// Enable dlmalloc as the global allocator when the "dlmalloc" feature is enabled and the "std" feature is not enabled.
#[cfg(feature = "dlmalloc")]
mod dlmalloc {
    use dlmalloc::GlobalDlmalloc;

    #[global_allocator]
    static ALLOC: GlobalDlmalloc = GlobalDlmalloc;
}

// Configure a panic handler that aborts the process when a panic occurs, which is suitable for a no_std environment.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::process::abort_immediate();
}