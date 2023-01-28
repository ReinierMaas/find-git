use colored::Colorize;
use git2::{BranchType, Repository, Status};
use std::env;
use std::path::PathBuf;
use std::{
    io,
    io::{Error, ErrorKind},
};
use walkdir::{DirEntry, WalkDir};

// Get git status
fn git_status() -> Status {
    let mut cmp_status = Status::all();
    cmp_status.remove(Status::IGNORED);
    cmp_status
}

// Error conversion
fn from_git_error(e: git2::Error) -> Error {
    Error::new(ErrorKind::Other, e)
}

// DirEntry is a '.git' folder
fn is_git(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s == ".git")
        .unwrap_or(false)
}

fn main() -> io::Result<()> {
    let cmp_status = git_status();
    let current_dir = env::current_dir()?;
    let walker = WalkDir::new(current_dir).into_iter();
    let mut repos: Vec<PathBuf> = vec![];

    println!("{}", "Found repositories:".bold().blue());
    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(parent) = entry.path().parent() {
            if is_git(&entry) {
                let parent_display = format!("    {}", parent.display());
                println!("{}", parent_display.italic().green());
                repos.push(parent.to_owned());
            }
        }
    }

    println!("{}", "Repositories with uncommitted changes:".bold().blue());
    for parent in &repos {
        let repo = Repository::open(parent).map_err(from_git_error)?;
        let statuses = repo.statuses(None).map_err(from_git_error)?;
        if statuses
            .iter()
            .any(|status| status.status().intersects(cmp_status))
        {
            let parent = format!("    {}", parent.display());
            println!("{}", parent.italic().green());
            for status in statuses
                .iter()
                .filter(|s| s.path().is_some() && s.status().intersects(cmp_status))
            {
                println!("        {}", status.path().unwrap().cyan());
            }
        }
    }

    println!("{}", "Repositories with stashed changes:".bold().blue());
    for parent in &repos {
        let mut repo = Repository::open(parent).map_err(from_git_error)?;
        let mut stashes = vec![];
        repo.stash_foreach(|_, stash, _| {
            stashes.push(stash.to_owned());
            true
        })
        .map_err(from_git_error)?;
        if !stashes.is_empty() {
            let parent = format!("    {}", parent.display());
            println!("{}", parent.italic().green());
        }
        for stash in stashes {
            println!("        {}", stash.red());
        }
    }

    println!("{}", "Repositories with unpushed branches:".bold().blue());
    for parent in &repos {
        let repo = Repository::open(parent).map_err(from_git_error)?;
        let branches = repo
            .branches(Some(BranchType::Local))
            .map_err(from_git_error)?;
        let diff_branches: Vec<String> = branches
            .filter_map(|r| r.ok())
            .filter(|(branch, _)| {
                if let Ok(upstream) = branch.upstream() {
                    let upstream = upstream
                        .get()
                        .peel_to_tree()
                        .map_err(from_git_error)
                        .unwrap();
                    let branch = branch.get().peel_to_tree().map_err(from_git_error).unwrap();
                    let diff = repo
                        .diff_tree_to_tree(Some(&upstream), Some(&branch), None)
                        .map_err(from_git_error)
                        .unwrap();
                    diff.deltas().any(|_| true)
                } else {
                    true
                }
            })
            .map(|(branch, _)| {
                branch
                    .name()
                    .map_err(from_git_error)
                    .unwrap()
                    .unwrap_or("")
                    .to_owned()
            })
            .collect();
        if !diff_branches.is_empty() {
            let parent = format!("    {}", parent.display());
            println!("{}", parent.italic().green());
        }
        for branch in diff_branches {
            println!("        {}", branch.yellow());
        }
    }
    Ok(())
}
