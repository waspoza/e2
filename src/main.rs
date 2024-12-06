use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
//use rayon_core;

mod atomic;
mod dents;

fn main() -> Result<(), std::io::Error> {
    let arena = atomic::Arena::new(8_000_000);
    let stack = atomic::Stack::<dents::Entry>::new(101);
    let mut dirs = vec![];

    let path = Arc::new(PathBuf::from("/home/piotr/Applications"));
    dirs.push(path);
    let path2 = Arc::new(PathBuf::from("/home/piotr/Pictures"));
    dirs.push(path2);

    //dbg!(ptr);
    //dbg!(dirfd);
    //dbg!(res);
    //dbg!(d_reclen);
    //dbg!(name);
    //dbg!(n);

    dirs.into_par_iter().for_each(|dir| {
        let res = dents::scandir(&arena, &stack, dir);
        assert!(res.is_ok(), "Error: {res:?}");
    });

    //dbg!(unsafe { *ptr.add(66) });
    let v = stack.into_vec();
    dbg!(v.len(), v.capacity());
    dbg!(v);

    Ok(())
}
