#[link(wasm_import_module = "shared_helper")]
unsafe extern "C" {
    #[link_name = "shared_load_word"]
    unsafe fn __shared_load_word(offset: i32) -> i32;

    #[link_name = "shared_store_word"]
    unsafe fn __shared_store_word(offset: i32, value: i32);

    #[link_name = "shared_copyfrom"]
    unsafe fn __shared_copyfrom(dest_offset: i32, src_offset: i32, size: i32);

    #[link_name = "shared_copyto"]
    unsafe fn __shared_copyto(dest_offset: i32, src_offset: i32, size: i32);

    #[link_name = "shared_atomic_wait"]
    unsafe fn __shared_atomic_wait(offset: i32, expected: i32, timeout_ns: i64) -> i32;

    #[link_name = "shared_atomic_store"]
    unsafe fn __shared_atomic_store(offset: i32, value: i32);
}

/// Load a word from the shared memory at the given offset.
#[inline(always)]
pub fn load_word(offset: i32) -> i32 {
    assert!(offset % 4 == 0, "Offset must be a multiple of 4");
    assert!(offset >= 0, "Offset must be non-negative");
    assert!(offset < 65536, "Offset must be less than 65536 (64KB - page size)");
    // SAFETY: Offset is within the bounds of the shared memory and properly aligned.
    unsafe { __shared_load_word(offset) }
}

/// Store a word to the shared memory at the given offset.
#[inline(always)]
pub fn store_word(offset: i32, value: i32) {
    assert!(offset % 4 == 0, "Offset must be a multiple of 4");
    assert!(offset >= 0, "Offset must be non-negative");
    assert!(offset < 65536, "Offset must be less than 65536 (64KB - page size)");
    // SAFETY: Offset is within the bounds of the shared memory and properly aligned.
    unsafe { __shared_store_word(offset, value) }
}

/// Copy a block of memory from the shared memory in buffer
#[inline(never)]
pub fn copyfrom(src_offset: i32, buf: &mut [u8]) -> usize {
    assert!(src_offset >= 0, "Source offset must be non-negative");
    assert!(src_offset < 65536, "Source offset must be less than 65536 (64KB - page size)");
    if buf.is_empty() {
        return 0;
    }
    let max_len = 65536 - src_offset as usize;
    let copy_len = buf.len().min(max_len);
    let buf_start = buf.as_ptr() as i32;
    // SAFETY: Enough items in the buffer, this points correctly to the last item.
    let buf_last = unsafe { buf.as_ptr().add(copy_len - 1) } as i32;
    // On wasm target, as the stack is emulated and grows downwards, 
    // the buffer last item may be located at a lower address than the start offset.
    // In this case, we need to copy backwards, otherwise we may write stupid things elsewhere.
    let dest_offset = buf_start.min(buf_last);

    // SAFETY: Source offset and buffer length are within the bounds of the shared memory.
    unsafe { __shared_copyfrom(dest_offset, src_offset, copy_len as i32) }
    copy_len
}

/// Copy a block of memory from the buffer to the shared memory
#[inline(never)]
pub fn copyto(dest_offset: i32, buf: &[u8]) -> usize {
    assert!(dest_offset >= 0, "Destination offset must be non-negative");
    assert!(dest_offset < 65536, "Destination offset must be less than 65536 (64KB - page size)");
    if buf.is_empty() {
        return 0;
    }
    let max_len = 65536 - dest_offset as usize;
    let copy_len = buf.len().min(max_len);
    let buf_start = buf.as_ptr() as i32;
    // SAFETY: Enough items in the buffer, this points correctly to the last item.
    let buf_last = unsafe { buf.as_ptr().add(copy_len - 1) } as i32;
    // On wasm target, as the stack is emulated and grows downwards, 
    // the buffer last item may be located at a lower address than the start offset.
    // In this case, we need to copy backwards, otherwise we may write stupid things elsewhere.
    let src_offset = buf_start.min(buf_last);
    // SAFETY: Destination offset and buffer length are within the bounds of the shared memory.

    unsafe { __shared_copyto(dest_offset, src_offset, copy_len as i32) }
    copy_len
}

/// Wait on an address of the shared memory until it changes from the expected value or timeouts.
#[inline(always)]
pub fn atomic_wait(offset: i32, expected: i32, timeout_ns: i64) -> i32 {
    assert!(offset % 4 == 0, "Offset must be a multiple of 4");
    assert!(offset >= 0, "Offset must be non-negative");
    assert!(offset < 65536, "Offset must be less than 65536 (64KB - page size)");
    // SAFETY: Offset is within the bounds of the shared memory and properly aligned.
    unsafe { __shared_atomic_wait(offset, expected, timeout_ns) }
}

/// Store a word to the shared memory at the given offset.
#[inline(always)]
pub fn atomic_store(offset: i32, value: i32) {
    assert!(offset % 4 == 0, "Offset must be a multiple of 4");
    assert!(offset >= 0, "Offset must be non-negative");
    assert!(offset < 65536, "Offset must be less than 65536 (64KB - page size)");
    // SAFETY: Offset is within the bounds of the shared memory and properly aligned.
    unsafe { __shared_atomic_store(offset, value) }
}