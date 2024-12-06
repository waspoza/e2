use rayon::prelude::*;
use std::ffi::{c_char, CString};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::Arc;
//use rayon_core;

mod atomic;

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
struct Entry {
    dir: Arc<PathBuf>,
    name: &'static str,
    date: usize,
}

fn main() -> Result<(), std::io::Error> {
    let arena = atomic::Arena::new(8_000_000);
    let stack = atomic::Stack::<Entry>::new(101);
    let mut dirents = vec![];

    let path = Arc::new(PathBuf::from("/home/piotr"));
    let dirfd = unsafe {
        let path = CString::new(path.as_os_str().as_bytes())?;
        try_posix_fn!(libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
        ))
    };

    let buf_size = 1024;
    loop {
        let ptr = arena.alloc(buf_size);
        let mut idx = 0;

        let bytes_read = loop {
            let res = unsafe { libc::syscall(libc::SYS_getdents64, dirfd, ptr, buf_size) };

            if res >= 0 {
                break res as usize;
            }

            // `res` contains an error. Retry if it is `EINTR`.
            let error = -res as i32;
            if error != libc::EINTR {
                return Err(io::Error::from_raw_os_error(error));
            }
        };
        dbg!(bytes_read);
        if bytes_read == 0 {
            break; // no more entries
        }
        loop {
            let dirent: *const libc::dirent64 = unsafe { ptr.add(idx).cast() };

            let d_reclen = unsafe { (*dirent).d_reclen as usize };
            //let name = unsafe { CStr::from_ptr((*dirent).d_name.as_ptr()) };
            let d_name: *const c_char = unsafe { (*dirent).d_name.as_ptr() };
            let namelen = unsafe { libc::strlen(d_name) };
            let slice: &[u8] = unsafe { std::slice::from_raw_parts(d_name.cast(), namelen) };
            let name = unsafe { std::str::from_utf8_unchecked(slice) };

            let entry = Entry {
                dir: path.clone(),
                name,
                date: 0,
            };
            dbg!(&entry);
            dirents.push(entry);

            idx += d_reclen;
            dbg!(idx);
            if idx >= bytes_read {
                break;
            }
        }
    }

    //dbg!(ptr);
    //dbg!(dirfd);
    //dbg!(res);
    //dbg!(d_reclen);
    //dbg!(name);
    //dbg!(n);

    dirents.into_par_iter().for_each(|x| {
        //let old = idx.fetch_add(1, Ordering::Relaxed);
        //let a = aptr.load(Ordering::Relaxed);
        //unsafe {
        //    *a.add(old) = old as u8;
        //}

        //arena.write(x + 1024, x as u8);

        //dbg!(&entry);
        stack.push(x);
        //dbg!(&sptr.get());
    });

    //dbg!(unsafe { *ptr.add(66) });
    let v = stack.into_vec();
    dbg!(v.len(), v.capacity());
    dbg!(v);

    Ok(())
}
