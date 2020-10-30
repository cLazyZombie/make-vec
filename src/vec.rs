use std::ptr::{NonNull, self};
use std::alloc::{self, Layout };
use std::mem;
use std::ops::{Deref, DerefMut};
pub struct Vec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() != 0, "no zero type");
        Vec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self) {
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

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap { self.grow(); }
        unsafe {
            ptr::write(self.ptr.as_ptr().offset(self.len as isize), elem);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T>{
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        let value;
        unsafe {
            value = self.ptr.as_ptr().offset(self.len as isize).read();
        }
        Some(value)
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        if self.len == self.cap { self.grow(); }
        unsafe {
            // check overflow index
            if index > self.len { panic!("overflow {}", index)}

            let move_count = self.len - index;
            if move_count > 0 {
                self.ptr.as_ptr().add(index).copy_to(self.ptr.as_ptr().add(index+1), move_count);    
            }
            self.ptr.as_ptr().add(index).write(elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        unsafe {
            if self.len == 0 { panic!("empty"); }
            if index >= self.len { panic!("out of index"); }

            let value = self.ptr.as_ptr().add(index).read();

            let move_count = self.len - index - 1;
            if move_count > 0 {
                self.ptr.as_ptr().add(index+1).copy_to(self.ptr.as_ptr().add(index), move_count);
            }
            
            self.len -= 1;

            value
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            while let Some(_) = self.pop() {}

            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}


impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_vec() {
        let mut v =  Vec::<i32>::new();

        for n in 1..=100 {
            v.push(n);
        }

        for n in (1..=100).rev() {
            assert_eq!(v.pop(), Some(n));
        }

        for n in 1..=100 {
            v.push(n);
        }

        for n in (1..=100).rev() {
            assert_eq!(v.pop(), Some(n));
        }

        assert_eq!(v.pop(), None);
        drop(v);
    }

    #[test]
    fn test_deref() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(&*v, &[1, 2, 3]);
    }

    #[test]
    fn test_deref_mut() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(&mut *v, &mut [1, 2, 3]);
    }

    #[test]
    fn test_insert() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);

        v.insert(0, 10);
        assert_eq!(&*v, &[10, 1, 2]);

        v.insert(1, 20);
        assert_eq!(&*v, &[10, 20, 1, 2]);

        v.insert(4, 30);
        assert_eq!(&*v, &[10, 20, 1, 2, 30]);
    }


    #[test]
    fn test_insert_when_empty() {
        let mut v = Vec::new();
        v.insert(0, 1);
        assert_eq!(&*v, &[1]);
    }

    #[test]
    #[should_panic]
    fn test_insert_out_of_bound() {
        let mut v = Vec::new();
        v.push(1);
        v.insert(2, 2);
    }

    #[test]
    fn test_remove() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);

        // remove middle
        assert_eq!(v.remove(1), 2);
        assert_eq!(&*v, &[1, 3, 4]);

        // remove last
        assert_eq!(v.remove(2), 4);
        assert_eq!(&*v, &[1, 3]);

        // remove first
        assert_eq!(v.remove(0), 1);
        assert_eq!(&*v, &[3]);

        // remove single
        assert_eq!(v.remove(0), 3);
        assert_eq!(&*v, &[]);
    }

    #[test]
    #[should_panic]
    fn test_remove_when_empty() {
        let mut v = Vec::<i32>::new();
        v.remove(0);
    }

    #[test]
    #[should_panic]
    fn test_remove_out_of_bound() {
        let mut v = Vec::<i32>::new();
        v.push(1);
        v.remove(1);
    }
}