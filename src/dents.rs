use crate::atomic;
use std::ffi::{c_char, CString};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::Arc;

macro_rules! try_posix_fn {
    ($call:expr) => {
        loop {
            let res = $call;

            if res != -1 {
                break res;
            }

            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(());
            }
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    };
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Entry {
    dir: Arc<PathBuf>,
    name: &'static str,
    pub date: usize,
}

pub fn scandir(
    arena: &atomic::Arena,
    stack: &atomic::Stack<Entry>,
    dir: &Arc<PathBuf>,
) -> Result<(), io::Error> {
    let dirfd = unsafe {
        let path = CString::new(dir.as_os_str().as_bytes())?;
        try_posix_fn!(libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_DIRECTORY,
        ))
    };
    let buf_size = 4096;
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
        //dbg!(bytes_read);
        if bytes_read == 0 {
            break; // no more entries
        }
        loop {
            let dirent: *const libc::dirent64 = unsafe { ptr.add(idx).cast() };

            let d_reclen = unsafe { (*dirent).d_reclen as usize };
            let d_name: *const c_char = unsafe { (*dirent).d_name.as_ptr() };
            let namelen = unsafe { libc::strlen(d_name) };
            let slice: &[u8] = unsafe { std::slice::from_raw_parts(d_name.cast(), namelen) };
            let name = unsafe { std::str::from_utf8_unchecked(slice) };

            // get only dovecot mail files which fist 10 chars are unix timestamp followed by a dot
            if matches!(name.chars().nth(10), Some('.')) {
                let entry = Entry {
                    dir: dir.clone(),
                    name,
                    date: name[0..10].parse().unwrap(),
                };
                stack.push(entry);
            }

            idx += d_reclen;
            if idx >= bytes_read {
                break;
            }
        }
    }
    Ok(())
}
