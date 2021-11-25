#![feature(is_symlink)]
#![feature(core_intrinsics)]
use crossbeam::{atomic::AtomicCell, channel};
use git2::{self, Repository};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::{
    fs, io, path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread,
};

fn visit_dirs(dir: &path::Path, cb: &dyn Fn(fs::DirEntry)) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&path, cb)?;
        }
        cb(entry);
    }
    Ok(())
}

fn read_file(path: &path::Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn main() {
    let start = std::time::Instant::now();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let count = Arc::new(AtomicU32::new(0));
    let dup = count.clone();
    let (s, r) = channel::unbounded();

    let handle = thread::spawn(move || {
        visit_dirs(path::Path::new("/home/semesse/code/workspace/"), &|entry| {
            dup.fetch_add(1, Ordering::Relaxed);
            s.send(entry).unwrap();
        })
        .ok();
    });

    r.iter().for_each(|entry| {
        // read_file(&entry.path()).ok();
        entry.metadata().unwrap().size();
    });

    println!("{} {:?}", count.load(Ordering::Relaxed), start.elapsed());
    // println!("{} {:?}", start.elapsed());
}
