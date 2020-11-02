use std::ptr;
use std::mem;
use std::ops::{Deref, DerefMut};

use crate::raw_vec::RawVec;

pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() != 0, "no zero type");
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.buf.cap { self.buf.grow(); }
        unsafe {
            ptr::write(self.buf.ptr.as_ptr().offset(self.len as isize), elem);
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
            value = self.buf.ptr.as_ptr().offset(self.len as isize).read();
        }
        Some(value)
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        if self.len == self.buf.cap { self.buf.grow(); }
        unsafe {
            // check overflow index
            if index > self.len { panic!("overflow {}", index)}

            let move_count = self.len - index;
            if move_count > 0 {
                self.buf.ptr.as_ptr().add(index).copy_to(self.buf.ptr.as_ptr().add(index+1), move_count);    
            }
            self.buf.ptr.as_ptr().add(index).write(elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        unsafe {
            if self.len == 0 { panic!("empty"); }
            if index >= self.len { panic!("out of index"); }

            let value = self.buf.ptr.as_ptr().add(index).read();

            let move_count = self.len - index - 1;
            if move_count > 0 {
                self.buf.ptr.as_ptr().add(index+1).copy_to(self.buf.ptr.as_ptr().add(index), move_count);
            }
            
            self.len -= 1;

            value
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let buf = ptr::read(&self.buf);
            let len = self.len;
    
            mem::forget(self);

            IntoIter {
                start: buf.ptr.as_ptr(),
                end: buf.ptr.as_ptr().add(len),
                _buf: buf,
            }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.buf.ptr.as_ptr(), self.len)
        }
    }
}


impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.buf.ptr.as_ptr(), self.len)
        }
    }
}

pub struct IntoIter<T> {
    _buf: RawVec<T>,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let value = self.start.read();
                self.start = self.start.add(1);
                Some(value)
            }
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        while let Some(_) = self.next() {}
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