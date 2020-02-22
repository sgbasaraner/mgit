use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "The stupid content tracker")]
enum Command {
    Init
}

fn main() {
    let command = Command::from_args();
    println!("{:#?}", command);
}
