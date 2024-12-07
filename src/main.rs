use rayon::prelude::*;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::sync::Arc;
//use rayon_core;

mod atomic;
mod dents;

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
    dbg!(needle, check_number, print_all);
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

    // let path = Arc::new(PathBuf::from(
    //     "/nexus/dovecot/data/mail/baza@amnezja.pl/Maildir/new",
    // ));

    maildirs.par_iter().for_each(|dir| {
        let res = dents::scandir(&arena, &stack, &dir);
        assert!(res.is_ok(), "Error in dir: {dir:?} - {res:?}");
    });

    let mut files = stack.into_vec();
    dbg!(files.len(), files.capacity());
    dbg!(arena.allocated());

    files.par_sort_unstable_by(|a, b| b.date.cmp(&a.date));

    for file in files.iter().take(5) {
        dbg!(file);
    }

    Ok(())
}
