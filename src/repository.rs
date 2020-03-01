use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::path::Path;
use ini::Ini;

pub struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
    conf: Ini
}

#[derive(Debug)]
enum RepoInitError {
    NotAGitRepository,
    ConfigFileMissing,
    UnsupportedVersion
}

impl std::fmt::Display for RepoInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.description())
    }
}

impl Error for RepoInitError {
    fn description(&self) -> &str {
        match self {
            RepoInitError::NotAGitRepository => "Not a git repository.",
            RepoInitError::ConfigFileMissing => "Configuration file missing.",
            RepoInitError::UnsupportedVersion => "Unsupported repository version.",
        }
    }
}

impl Repository {

    pub fn new(path: PathBuf) -> Repository {
        // path should either be empty or a directory
        if path.exists() && !path.is_dir() {
            panic!("{:#?} is not a directory!", path);
        } else {
            fs::create_dir(&path).unwrap_err();
        }
        
        let repo = Repository::naive_from_path(path.to_path_buf());

        // create git directories
        assert!(repo_dir(&repo, &vec!["branches"], true).is_some());
        assert!(repo_dir(&repo, &vec!["objects"], true).is_some());
        assert!(repo_dir(&repo, &vec!["refs", "tags"], true).is_some());
        assert!(repo_dir(&repo, &vec!["refs", "heads"], true).is_some());

        // .git/description
        let description = "Unnamed repository; edit this file 'description' to name the repository.\n";
        write_repo_file(&repo, "description", description);

        // .git/HEAD
        let head = "ref: refs/heads/master\n";
        write_repo_file(&repo, "HEAD", head);

        // config
        let mut config_bytes: Vec<u8> = vec![];
        assert!(repo.conf.write_to(&mut config_bytes).is_ok());
        write_repo_file(&repo, "config", &std::str::from_utf8(&config_bytes).unwrap());

        return repo;
    }

    fn from_path(path: PathBuf) -> Result<Repository, RepoInitError> {
        let repo = Repository::naive_from_path(path.to_path_buf());
        if !repo.gitdir.is_dir() { return Err(RepoInitError::NotAGitRepository); }

        let config_file = repo_file(&repo, &vec![path.join("config").to_path_buf()], false);
        if config_file.is_none() { return Err(RepoInitError::ConfigFileMissing); }

        let parsed_config = Ini::load_from_file(&config_file.unwrap());
        let config = match parsed_config {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                return Err(RepoInitError::ConfigFileMissing);
            },
        };

        let version = config.section(Some("core")).and_then(|x| x.get("repositoryformatversion").and_then(|y| y.parse::<i32>().ok() ));
        if version.is_none() || version.unwrap() != 0 { return Err(RepoInitError::UnsupportedVersion); }

        return Ok(Repository {
            worktree: repo.worktree.to_path_buf(),
            gitdir: repo.gitdir.to_path_buf(),
            conf: config
        });
    }

    fn naive_from_path(path: PathBuf) -> Repository {
        let gitdir = path.join(".git/");
        
        Repository {
            worktree: path.to_path_buf(),
            gitdir: gitdir.to_path_buf(),
            conf: repo_default_config()
        }
    }
}

fn write_repo_file(repo: &Repository, file_name: &str, content: &str) {
    let file_path = repo_file(&repo, &vec![file_name], false);
    assert!(file_path.is_some());

    let mut file = match File::create(&file_path.unwrap()) {
        Err(_) => panic!("couldn't create {} file", file_name),
        Ok(file) => file,
    };
    assert!(file.write_all(content.as_bytes()).is_ok());
}

fn repo_default_config() -> Ini {
    let mut ini = Ini::new();
    ini.with_section(Some("core"))
        .set("repositoryformatversion", "0")
        .set("filemode", "false")
        .set("bare", "false");
    ini
}

fn repo_path<P: AsRef<Path>>(repo: &Repository, paths: &Vec<P>) -> PathBuf {
    let mut path = repo.gitdir.clone();
    for p in paths {
        path.push(p);
    }
    path
}

fn repo_file<P: AsRef<Path>>(repo: &Repository, paths: &Vec<P>, mkdir: bool) -> Option<PathBuf> {
    let all_except_last = paths.into_iter().take(paths.len() - 1).collect();
    if repo_dir(repo, &all_except_last, mkdir).is_some() {
        return Some(repo_path(repo, paths))
    } else {
        return None
    }
}

fn repo_dir<P: AsRef<Path>>(repo: &Repository, paths: &Vec<P>, mkdir: bool) -> Option<PathBuf> {
    let repo_path = repo_path(repo, paths);
    if repo_path.exists() {
        if repo_path.is_dir() {
            return Some(repo_path);
        } else {
            panic!("Not a directory {:#?}", repo_path);
        }
    }

    if !mkdir { return None; }

    return fs::create_dir_all(&repo_path).ok().and_then(|_| { Some(repo_path) });
}

pub fn repo_find(path: PathBuf) -> Option<Repository> {
    if path.join(".git/").is_dir() { 
        return match Repository::from_path(path) {
            Ok(repo) => Some(repo),
            Err(e) => {
                println!("{}", e);
                None
            },
        }
    }

    let parent = path.parent();
    return match parent {
        Some(p) => repo_find(p.to_path_buf()),
        None => None,
    }
}

pub fn repo_find_or_panic(path: PathBuf) -> Repository {
    return repo_find(path).expect("No git directory.");
}