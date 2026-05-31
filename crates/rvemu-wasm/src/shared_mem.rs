#[link(wasm_import_module = "shared_helper")]
unsafe extern "C" {
    #[link_name = "shared_load_word"]
    unsafe fn shared_load_word(offset: i32) -> i32;

    #[link_name = "shared_store_word"]
    unsafe fn shared_store_word(offset: i32, value: i32);

    #[link_name = "shared_copyfrom"]
    unsafe fn shared_copyfrom(dest_offset: i32, src_offset: i32, size: i32);

    #[link_name = "shared_copyto"]
    unsafe fn shared_copyto(dest_offset: i32, src_offset: i32, size: i32);
}

/// Load a word from the shared memory at the given offset.
#[inline(never)]
pub fn load_word(offset: i32) -> i32 {
    assert!(offset % 4 == 0, "Offset must be a multiple of 4");
    assert!(offset >= 0, "Offset must be non-negative");
    assert!(offset < 65536, "Offset must be less than 65536 (64KB - page size)");
    // SAFETY: Offset is within the bounds of the shared memory and properly aligned.
    unsafe { shared_load_word(offset) }
}

/// Store a word to the shared memory at the given offset.
#[inline(never)]
pub fn store_word(offset: i32, value: i32) {
    assert!(offset % 4 == 0, "Offset must be a multiple of 4");
    assert!(offset >= 0, "Offset must be non-negative");
    assert!(offset < 65536, "Offset must be less than 65536 (64KB - page size)");
    // SAFETY: Offset is within the bounds of the shared memory and properly aligned.
    unsafe { shared_store_word(offset, value) }
}

/// Copy a block of memory from the shared memory in buffer
pub fn copyfrom(src_offset: i32, buf: &mut [i32]) -> usize {
    assert!(src_offset >= 0, "Source offset must be non-negative");
    assert!(src_offset < 65536, "Source offset must be less than 65536 (64KB - page size)");
    let max_len = 65536 - src_offset as usize;
    let copy_len = buf.len().min(max_len);
    // SAFETY: Source offset and buffer length are within the bounds of the shared memory.
    unsafe { shared_copyfrom(buf.as_mut_ptr() as i32, src_offset, copy_len as i32) }
    copy_len
}

/// Copy a block of memory from the buffer to the shared memory
pub fn copyto(dest_offset: i32, buf: &[i32]) -> usize {
    assert!(dest_offset >= 0, "Destination offset must be non-negative");
    assert!(dest_offset < 65536, "Destination offset must be less than 65536 (64KB - page size)");
    let max_len = 65536 - dest_offset as usize;
    let copy_len = buf.len().min(max_len);
    // SAFETY: Destination offset and buffer length are within the bounds of the shared memory.
    unsafe { shared_copyto(dest_offset, buf.as_ptr() as i32, copy_len as i32) }
    copy_len
}