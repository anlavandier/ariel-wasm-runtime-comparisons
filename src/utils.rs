
#[macro_export]
macro_rules! benchmark_name {
    () => {
        env!("BENCHMARK")
    };
}

#[macro_export]
macro_rules! benchmark_file {
    () => {
        env!("BENCHMARK_PATH")
    }
}

#[cfg(all(feature = "wamr", feature = "embench-1"))]
pub use alloc::SendCell;

#[cfg(feature = "wamr")]
mod alloc {
    // Allocator bindings required by WAMR
    use core::ffi::{c_uint, c_void};
    use core::alloc::Layout;
    use core::cell::{RefCell, RefMut};
    use core::ptr;

    extern crate alloc;
    use alloc::collections::btree_map::BTreeMap;

    use ariel_os::debug::log::{debug, error};

    // C doesn't ask for a specific alignement, only for a size. We fix it to 16.
    const C_ALIGN: usize = 16;
    // Rust Allocations APIs require a layout at reallocation and deallocation which C doesn't communicate.
    // For this reason, we need to save the layout information of each C allocation.
    static C_ALLOCATIONS: SendCell<BTreeMap<*mut c_void, Layout>> = SendCell::new(BTreeMap::new());

    pub struct SendCell<T> {
        inner: RefCell<T>
    }

    /// SAFETY:
    /// Our execution environment is single threaded
    unsafe impl<T> Send for SendCell<T> {}
    unsafe impl<T> Sync for SendCell<T> {}

    impl<T> SendCell<T> {
        pub const fn new(value: T) -> Self {
            SendCell { inner: RefCell::new(value) }
        }
        pub fn borrow_mut(&self) -> RefMut<'_, T> {
            self.inner.borrow_mut()
        }
    }


    #[unsafe(no_mangle)]
    extern "C" fn ariel_malloc(size: c_uint) -> *mut c_void {
        // debug!("[WAMR] Alloc {:?} bytes", size as usize);
        let layout = Layout::from_size_align(size.try_into().unwrap(), C_ALIGN);
        if let Ok(layout) = layout {
            let c_ptr = unsafe { alloc::alloc::alloc(layout) as *mut core::ffi::c_void };
            if c_ptr.is_null() {
                error!("[WAMR] not enough space left");
                let total_in_wamr = C_ALLOCATIONS.borrow_mut().values().fold(0, |acc, lay| { acc + lay.size() });
                error!("[WAMR] total of {} bytes in allocations", total_in_wamr);
                panic!("[WAMR] Failed allocation")
            }
            assert!(C_ALLOCATIONS.borrow_mut().insert(c_ptr, layout).is_none(), "[WAMR] Somehow already created this allocation...");
            return c_ptr;
        }
        else {
            panic!("[WAMR] failed alloc");
        }
    }

    #[unsafe(no_mangle)]
    extern "C" fn ariel_realloc(addr: *mut c_void, size: c_uint) -> *mut c_void {
        if addr.is_null() {
            debug!("[WAMR] trying to realloc a null ptr...");
            return ptr::null_mut();
        }
        if size == 0 { panic!("[WAMR] UB"); }
        let old_layout = C_ALLOCATIONS.borrow_mut().remove(&addr);
        if old_layout.is_none() {
            panic!("[WAMR] Unknown allocation");
        }
        let old_layout = old_layout.unwrap();
        let new_addr = unsafe {
            alloc::alloc::realloc(addr as *mut u8, old_layout, size.try_into().unwrap()) as *mut core::ffi::c_void
        };
        if new_addr.is_null() {
            panic!("[WAMR] failed realloc")
        }
        C_ALLOCATIONS.borrow_mut().insert(new_addr, Layout::from_size_align(size as usize, old_layout.align()).unwrap());
        return new_addr
    }

    #[unsafe(no_mangle)]
    extern "C" fn ariel_free(addr: *mut c_void) {
        if addr.is_null() {
            debug!("[WAMR] freeing a null pointer...");
        }
        let old_layout = C_ALLOCATIONS.borrow_mut().remove(&addr);

        if old_layout.is_none() {
            panic!("[WAMR] unknown allocation");
        }
        let old_layout = old_layout.unwrap();
        // debug!("[WAMR] freeing {} bytes", old_layout.size());
        unsafe { alloc::alloc::dealloc(addr as *mut u8, old_layout) };
    }
}
