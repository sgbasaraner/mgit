use std::fs;
use std::path::PathBuf;
use std::path::Path;
use structopt::StructOpt;
use ini::Ini;

#[derive(StructOpt, Debug)]
#[structopt(about = "The stupid content tracker")]
enum Command {
    Init {
        #[structopt(parse(from_os_str), default_value = ".")]
        path: PathBuf
    }
}

fn main() {
    let command = Command::from_args();
    match command {
        Command::Init { path } => init(path),
    }
}

fn init(path: PathBuf) {
    println!("{:#?}", path);
}

struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
    conf: Option<Ini>
}

impl Repository {
    fn new(path: PathBuf) -> Repository {
        let gitdir = path.join(".git");
        if !gitdir.is_dir() {
            panic!("Not a Git repository {:#?}", path);
        }

        let repo = Repository {
            worktree: path,
            gitdir: gitdir,
            conf: None
        };

        // # Read configuration file in .git/config
        // self.conf = configparser.ConfigParser()
        // cf = repo_file(self, "config")

        // if cf and os.path.exists(cf):
        //         self.conf.read([cf])
        // elif not force:
        //     raise Exception("Configuration file missing")

        // if not force:
        //     vers = int(self.conf.get("core", "repositoryformatversion"))
        //     if vers != 0:
        //         raise Exception("Unsupported repositoryformatversion %s" % vers)


        // TODO: implement
        let conf = match Ini::load_from_file(&path) {
            Ok(config) => config,
            Err(_) => panic!("Configuration file missing")
        };

        Repository {
            worktree: path,
            gitdir: gitdir,
            conf: conf
        }
    }
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

    return fs::create_dir(&repo_path).ok().and_then(|_| { Some(repo_path) });
}