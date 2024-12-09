use memchr::memmem;
use memmap2::Mmap;
use rayon::prelude::*;
use std::cmp;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::sync::Arc;

mod atomic;
mod dents;
mod email;

const USAGE: &str = "\
Usage: e [options] search_string

Options:

-a          Display all occurences.
-c number   Number of emails to search. (default: 300)
";

fn main() -> Result<(), std::io::Error> {
    let arena = atomic::Arena::new(8_000_000);
    let stack = atomic::Stack::<dents::Entry>::new(50_000);

    let mut print_all = false; // dont stop searching after first find
    let mut check_number = 300; // how many newest emails to search
    let mut needle = OsString::default();

    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("-a") => print_all = true,

            Some("-c") => {
                check_number = args
                    .next()
                    .as_ref()
                    .and_then(|a| a.to_str())
                    .and_then(|a| str::parse(a).ok())
                    .expect("Option -c needs integer argument.");
            }

            Some("-h" | "--help") => {
                eprint!("{}", USAGE);
                return Ok(());
            }

            _ => needle = arg.to_owned(),
        }
    }
    let needle = needle.to_str().expect("bad search string");

    let homes =
        fs::read_dir("/nexus/dovecot/data/mail/")?.collect::<Result<Vec<_>, io::Error>>()?;
    //let homes = fs::read_dir("/root/docker/dovecot/data/mail/")?.collect::<Result<Vec<_>, io::Error>>()?;
    let dirs = vec![
        "Maildir/new/",
        "Maildir/cur/",
        "Maildir/.Trash/new/",
        "Maildir/.Trash/cur/",
        "Maildir/.Junk/new/",
        "Maildir/.Junk/cur/",
    ];
    let mut maildirs = vec![];
    for mbox in homes {
        for dir in &dirs {
            let mut path = mbox.path();
            path.push(dir);
            maildirs.push(Arc::new(path));
        }
    }

    maildirs.par_iter().for_each(|dir| {
        let res = dents::scandir(&arena, &stack, &dir);
        assert!(res.is_ok(), "Error in dir: {dir:?} - {res:?}");
    });

    let mut files = stack.into_vec();
    println!(
        "stack: {}/{}, arena: {}/{}",
        files.len(),
        files.capacity(),
        arena.allocated(),
        arena.capacity()
    );

    files.par_sort_unstable_by(|a, b| b.date.cmp(&a.date));

    // for file in files.iter().take(5) {
    //     dbg!(file);
    // }

    let finder = memmem::Finder::new(needle);

    // par_iter is slower here
    let _found = files.iter().take(check_number).find(|entry| {
        let mut filename = (*entry.dir).clone();
        filename.push(entry.name);
        let file = fs::File::open(&filename);
        assert!(file.is_ok(), "Error in file: {entry:?} - {file:?}");
        let file = file.unwrap();

        let mmap = unsafe { Mmap::map(&file) };
        assert!(mmap.is_ok(), "Error in mmap: {mmap:?}");
        let mmap = mmap.unwrap();
        let len = cmp::min(mmap.len(), 4096);

        match finder.find(&mmap[..len]) {
            None => return false,
            Some(_) => {
                println!("{}", &filename.to_str().unwrap());
                if print_all {
                    return false;
                }
                email::display(&mmap[..]);
                println!("\n{}", &filename.to_str().unwrap());
                return true;
            }
        }
    });

    Ok(())
}
