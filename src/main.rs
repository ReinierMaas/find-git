use std::env;
use std::path::Path;
use std::{io,io::{Error, ErrorKind}};
use std::process::Command;
use walkdir::{DirEntry, WalkDir};

// Error conversion
fn from_utf8_error(e: std::string::FromUtf8Error) -> Error {
    Error::new(ErrorKind::Other, e)
}

// Commands
fn git() -> Command {
    Command::new("git")
}

fn exists_git_cmd() -> io::Result<()> {
    git().arg("--version").output().map(|_| ())
}

fn git_status<P: AsRef<Path>>(dir: P) -> io::Result<String> {
    git()
        .current_dir(dir)
        .arg("status")
        .output()
        .and_then(|o| {
            String::from_utf8(o.stdout)
                   .map_err(from_utf8_error)
        })
}

// DirEntry is a '.git' folder
fn is_git(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s == ".git")
         .unwrap_or(false)
}

fn main() -> io::Result<()> {
    exists_git_cmd()?;
    let current_dir = env::current_dir()?;
    let walker = WalkDir::new(current_dir).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(parent) = entry.path().parent() {
            if is_git(&entry) {
                println!("{}", parent.display());
                println!("{}", git_status(parent)?);
            }
        }
    }
    Ok(())
}
