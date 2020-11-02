use std::{alloc::{self, Layout}, mem, ptr::NonNull};

pub struct RawVec<T> {
    pub ptr: NonNull<T>,
    pub cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "not allow zero sized T");
        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    pub fn grow(&mut self) {
        unsafe {
            let elem_size = mem::size_of::<T>();

            let (new_cap, ptr) = if self.cap == 0 {
                let layout = Layout::array::<T>(1).unwrap();
                let ptr = alloc::alloc(layout);
                (1, ptr)
            } else {
                let new_cap = self.cap * 2;
                let old_num_bytes = self.cap * elem_size;
                let new_num_bytes = old_num_bytes * 2;
                //let layout = std::alloc::Layout::new::<T>();
                let layout = Layout::array::<T>(self.cap).unwrap();
                let ptr = alloc::realloc(self.ptr.as_ptr() as *mut _, layout, new_num_bytes);
                (new_cap, ptr)
            };

            if ptr.is_null() { panic!("oom"); }

            self.ptr = std::ptr::NonNull::<T>::new(ptr as *mut _).unwrap();
            self.cap = new_cap;
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}