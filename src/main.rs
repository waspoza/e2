use rayon::prelude::*;
use std::ffi::{c_char, CString};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
//use rayon_core;

const BUFSIZE: usize = 8_000_000;

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
struct SyncPtr(*mut u8);
unsafe impl Send for SyncPtr {}
unsafe impl Sync for SyncPtr {}
impl SyncPtr {
    fn write(&self, offset: usize, val: u8) {
        unsafe {
            *self.0.add(offset) = val;
        }
    }
    fn get(&self) -> *mut u8 {
        self.0
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut buf = Vec::<u8>::with_capacity(BUFSIZE);
    let ptr = buf.as_mut_ptr();
    let idx = AtomicUsize::new(0);
    let sptr = SyncPtr(ptr);

    let path = PathBuf::from("/home/piotr");
    let dirfd = unsafe {
        let path = CString::new(path.as_os_str().as_bytes())?;
        try_posix_fn!(libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
        ))
    };

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

    (0..100).into_par_iter().for_each(|_x| {
        let old = idx.fetch_add(1, Ordering::Relaxed);
        //let a = aptr.load(Ordering::Relaxed);
        //unsafe {
        //    *a.add(old) = old as u8;
        //}

        sptr.write(old, old as u8);
        //dbg!(&sptr.get());
    });

    dbg!(unsafe { *ptr.add(66) });

    Ok(())
}
