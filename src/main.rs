use std::path::PathBuf;
use structopt::StructOpt;

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
