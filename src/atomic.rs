use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Arena {
    buf: Vec<u8>,
    idx: AtomicUsize,
}
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}
impl Arena {
    pub fn new(capacity: usize) -> Self {
        Arena {
            buf: Vec::<u8>::with_capacity(capacity),
            idx: AtomicUsize::new(0),
        }
    }
    pub fn alloc(&self, size: usize) -> *const u8 {
        // round up size to be divisible by 8
        // to avoid misalignings
        let new_size;
        if size % 8 > 0 {
            new_size = size + 8 - (size % 8);
        } else {
            new_size = size;
        }
        let old = self.idx.fetch_add(new_size, Ordering::Relaxed);
        assert!(
            old + size <= self.buf.capacity(),
            "alloc size exceeded buf capacity"
        );
        unsafe { self.ptr().add(old) }
    }
    #[allow(dead_code)]
    pub fn write(&self, offset: usize, val: u8) {
        unsafe {
            let ptr = self.ptr().add(offset) as *mut u8;
            ptr.write(val);
        }
    }
    #[inline]
    fn ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }
    #[allow(dead_code)]
    pub fn allocated(&self) -> usize {
        self.idx.load(Ordering::Relaxed)
    }
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }
}

//#[derive(Debug)]
pub struct Stack<T> {
    buf: Vec<T>,
    idx: AtomicUsize,
}
unsafe impl<T> Send for Stack<T> {}
unsafe impl<T> Sync for Stack<T> {}
impl<T> Stack<T> {
    pub fn new(capacity: usize) -> Self {
        Stack {
            buf: Vec::<T>::with_capacity(capacity),
            idx: AtomicUsize::new(0),
        }
    }
    pub fn push(&self, item: T) {
        let old_idx = self.idx.fetch_add(1, Ordering::Relaxed);
        assert!(old_idx + 1 <= self.buf.capacity(), "stack full");
        unsafe {
            let ptr = self.buf.as_ptr().add(old_idx) as *mut T;
            ptr.write(item);
        }
    }
    pub fn into_vec(self) -> Vec<T> {
        let me = std::mem::ManuallyDrop::new(self);
        let len = me.idx.load(Ordering::Relaxed);
        let cap = me.buf.capacity();
        let ptr = me.buf.as_ptr() as *mut T;
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }
}
