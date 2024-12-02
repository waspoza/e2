use rayon::prelude::*;
use std::ffi::{c_char, CString};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
//use rayon_core;

macro_rules! try_posix_fn {
    ($call:expr) => {
        loop {
            let res = $call;

            if res != -1 {
                break res;
            }

            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    };
}
#[derive(Debug)]
struct Arena {
    buf: Vec<u8>,
    idx: AtomicUsize,
}
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}
impl Arena {
    fn new(capacity: usize) -> Self {
        Arena {
            buf: Vec::<u8>::with_capacity(capacity),
            idx: AtomicUsize::new(0),
        }
    }
    fn alloc(&self, size: usize) -> *const u8 {
        let old = self.idx.fetch_add(1, Ordering::Relaxed);
        if old + size > self.buf.capacity() {
            panic!("alloc size exceeded buf capacity")
        }
        unsafe { self.get().add(old) }
    }
    fn write(&self, offset: usize, val: u8) {
        unsafe {
            let ptr = self.get().add(offset) as *mut u8;
            *ptr = val;
        }
    }
    fn get(&self) -> *const u8 {
        self.buf.as_ptr()
    }
}

fn main() -> Result<(), std::io::Error> {

    let arena = Arena::new(8_000_000);

    let path = PathBuf::from("/home/piotr");
    let dirfd = unsafe {
        let path = CString::new(path.as_os_str().as_bytes())?;
        try_posix_fn!(libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
        ))
    };

    let ptr = arena.alloc(1024);
    let res = unsafe { libc::syscall(libc::SYS_getdents64, dirfd, ptr, 1024) };
    let dirent: *const libc::dirent64 = ptr.cast();

    let d_reclen = unsafe { (*dirent).d_reclen as usize };

    //let name = unsafe { CStr::from_ptr((*dirent).d_name.as_ptr()) };
    let d_name: *const c_char = unsafe { (*dirent).d_name.as_ptr() };
    let namelen = unsafe { libc::strlen(d_name) };
    let name: &[u8] = unsafe { std::slice::from_raw_parts(d_name.cast(), namelen) };
    let n = unsafe { std::str::from_utf8_unchecked(name) };

    dbg!(ptr);
    dbg!(dirfd);
    dbg!(res);
    dbg!(d_reclen);
    dbg!(name);
    println!("{}", n);

    (0..100).into_par_iter().for_each(|x| {
        //let old = idx.fetch_add(1, Ordering::Relaxed);
        //let a = aptr.load(Ordering::Relaxed);
        //unsafe {
        //    *a.add(old) = old as u8;
        //}

        arena.write(x, x as u8);
        //dbg!(&sptr.get());
    });

    dbg!(unsafe { *ptr.add(66) });

    Ok(())
}
