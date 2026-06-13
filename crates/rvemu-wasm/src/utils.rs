use alloc::format;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    /// Copy an object (also known as a memory region) from the wasm module's main memory to the host environment.
    /// 
    /// # Safety
    /// This function is unsafe because it involves raw pointers and memory manipulation. The caller must ensure
    /// that the provided pointer and length are valid. This function does not account for thread-safety, and 
    /// therefore the javascript should not run while anything is being done on that particular memory region. 
    /// 
    /// # Parameters
    /// 
    /// - `ptr`: A pointer to the start of the memory region in the wasm module's main memory.
    /// - `len`: The length of the memory region to copy.
    /// 
    /// # Returns
    /// 
    /// - `usize`: An handle (ie a pointer) to the copied, newly allocated object in the javascript host environment's memory. 
    ///            The caller must ensure that this handle is properly managed and freed when no longer needed to avoid memory leaks.
    ///            If the operation fails, this function may return a null pointer (0).
    #[link_name = "obj_copyout"]
    unsafe fn env__obj_copyout(ptr: usize, len: usize) -> usize;

    /// Copy an object (also known as a memory region) from the host environment to the wasm module's main memory.
    /// 
    /// # Safety
    /// This function is unsafe because it involves raw pointers and memory manipulation. The caller must ensure
    /// that the provided pointer and length are valid. This function does not account for thread-safety, and therefore
    /// the javascript should not run while anything is being done on that particular memory region.
    /// 
    /// # Parameters
    /// 
    /// - `ptr`: A pointer to the start of the memory region in the wasm module's main memory where the object will be copied to.
    /// - `len`: The length of the memory region to copy.
    /// - `handle`: A handle (ie a pointer) to the object in the javascript host environment's memory that will be copied to 
    ///             the wasm module's main memory. This handle must be valid and point to a memory region that is 
    ///             at least `len` bytes long.
    /// 
    /// # Returns
    /// 
    /// - `usize`: The number of bytes copied. This should be equal to `len` if the operation was successful.
    ///            0 would indicate a failure if the len was greater than 0.
    #[link_name = "obj_copyin"]
    unsafe fn env__obj_copyin(ptr: usize, len: usize, handle: usize) -> usize;

    /// Free an object (also known as a memory region) in the host environment.
    /// 
    /// # Safety
    /// This function is unsafe because the handle provided can be misused.
    /// It must absolutely point to a valid memory object that was previously allocated by the host environment.
    /// You should not call this function with a handle that has already been freed or with a handle that
    /// was not allocated by the host environment.
    /// 
    /// # Parameters
    /// 
    /// - `handle`: A handle (ie a pointer) to the object in the javascript host environment's memory that 
    ///             will be freed. This handle must be valid and point to a memory region that was previously 
    ///             allocated by the host environment.
    #[link_name = "obj_free"]
    unsafe fn env__obj_free(handle: usize);

    /// Print a string to the host environment's console.
    /// 
    /// # Safety
    /// This function is unsafe as it handles a raw handle to the host environment's memory.
    /// The caller must ensure that the handle is valid and points to a valid utf-8 string.
    /// 
    /// # Parameters
    /// 
    /// - `handle`: A handle (ie a pointer) to the string in the javascript host environment's memory 
    ///             that will be printed. This handle must be valid and point to a valid utf-8 string.
    #[link_name = "console_log"]
    unsafe fn env__console_log(handle: usize);
}

pub fn console_log(str: impl AsRef<str>) {
    // SAFETY: Except thread-safety, memory region is expected to be valid, read-only while
    //         this function is running, and the string is valid utf-8.
    let s = str.as_ref();
    let handle = unsafe { env__obj_copyout(s.as_ptr() as usize, s.len()) };
    if handle == 0 {
        // If the handle is 0, it indicates that the copy operation failed. 
        // This could be due to insufficient memory in the host environment or other reasons.
        // In this case, we simply return without logging anything.
        return;
    }
    // SAFETY: The handle is expected to be valid, and it points to a valid utf-8 string.
    unsafe { env__console_log(handle) };
    // SAFETY: The handle is expected to be valid, and it points to a memory region that was previously 
    //         allocated by the host environment.
    unsafe { env__obj_free(handle) };
}

/// A struct that represents a handle to a memory object in the host environment.
/// 
/// This struct does not manage the lifetime / life-cycle of the memory object it represents.
/// The user is responsible for ensuring that the handle is valid and that the memory object is properly managed 
/// (e.g., freed when no longer needed).
/// 
/// This struct is merely an helper to encapsulate the handle and its associated length, providing a convenient way to
/// pass around the handle and its metadata. So basically, treat it as a raw pointer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HostMemoryHandle {
    handle: usize,
    len: usize,
}

impl HostMemoryHandle {
    pub fn new(handle: usize, len: usize) -> Self {
        HostMemoryHandle { handle, len }
    }

    pub fn get_handle(&self) -> usize {
        self.handle
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn is_null(&self) -> bool {
        self.handle == 0
    }

    /// Frees the memory object in the host environment that this handle represents.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe because it involves raw pointers and memory manipulation. The caller must ensure
    /// that the handle is valid and points to a memory object that was previously allocated by the host environment.
    /// You should not call this function with a handle that has already been freed.
    pub unsafe fn free(self) {
        unsafe {
            env__obj_free(self.handle);
        }
    }

    /// Copies the memory object in the host environment that this handle represents into the provided destination slice.
    /// Note: The destination slice length must match the length of the memory object represented by this handle, 
    /// otherwise the copy won't happen.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe because it involves raw pointers and memory manipulation. The caller must ensure
    /// that the handle is valid and points to a memory object that was previously allocated by the host environment.
    /// You should not call this function with a handle that has already been freed.
    /// 
    /// # Returns
    /// 
    /// The number of bytes copied. This should be equal to the length of the destination slice if the operation was successful.
    pub unsafe fn copy_from_host(&self, dest: &mut [u8]) -> usize {
        if dest.len() != self.len {
            // If the destination slice length does not match the length of the memory object,
            // we cannot perform the copy operation. Return 0 to indicate failure.
            return 0;
        }
        let dest_start = dest.as_ptr() as usize;
        let dest_end = unsafe { dest.as_ptr().add(dest.len() - 1) } as usize;
        let dest_ptr = dest_start.min(dest_end);
        unsafe { env__obj_copyin(dest_ptr as usize, dest.len(), self.handle) }
    }

    /// Creates a new `HostMemoryHandle` from an indirect handle (ie a pointer) to a memory object in the host environment.
    /// 
    /// Since multivalue is not really available with Rust & wasm, we might get return a single usize, an handle to a memory
    /// object of 8 bytes (on wasm32), which contains a raw HostMemoryHandle struct. 
    /// 
    /// This function allows us to create a new `HostMemoryHandle` from that indirect handle.
    /// 
    /// # Safety
    /// 
    /// As other functions, this function is unsafe because it involves raw pointers and memory manipulation. The caller must
    /// ensure that the indirect handle is valid and points to a memory object that was previously allocated by the host environment.
    pub unsafe fn from_indirect(handle: usize) -> Self {
        let mut buf = [0u8; size_of::<HostMemoryHandle>()];
        let indir =  HostMemoryHandle::new(handle, size_of::<HostMemoryHandle>());
        let written = unsafe { indir.copy_from_host(&mut buf) };
        if written != size_of::<HostMemoryHandle>() {
            // If the number of bytes copied does not match the size of HostMemoryHandle,
            // we cannot create a valid HostMemoryHandle. Return a null handle to indicate failure.
            console_log("HostMemoryHandle::from_indirect - written is not size of HostMemoryHandle");
            return HostMemoryHandle { handle: 0, len: 0 };
        }
        let handle : usize;
        let len : usize;
        console_log(format!("HostMemoryHandle::from_indirect - got {:?}", &buf));
        handle = usize::from_le_bytes(buf[0..size_of::<usize>()].try_into().unwrap());
        len = usize::from_le_bytes(buf[size_of::<usize>()..2*size_of::<usize>()].try_into().unwrap());
        HostMemoryHandle { handle, len }
    }
}

macro_rules! extract_word {
    ($buf:expr, $index:expr) => {
        ($buf[$index*4] as u32) 
        | (($buf[$index*4 + 1] as u32) << 8) 
        | (($buf[$index*4 + 2] as u32) << 16) 
        | (($buf[$index*4 + 3] as u32) << 24)
    };
}

pub(crate) use extract_word;