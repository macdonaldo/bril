use serde_json::Result;
use std::fs;
use structopt::StructOpt;

mod object;
use object::*;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let data = fs::read_to_string(&args.path).expect("Unable to read file");

    let deserialized: Result<Program> = serde_json::from_str(&data);
    match deserialized {
        Ok(p) => {
            for f in &p.functions {
                let basic_blocks = f.get_basic_blocks();
                let (successors, _) = f.get_edges(&basic_blocks);
                println!("add count: {}", f.count_add_ops());
                f.cfg_dot(&basic_blocks, &successors);
            }
        }
        Err(e) => println!("{:?}", e),
    }
}
