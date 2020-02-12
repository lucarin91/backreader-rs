extern crate backreader;
use std::env;
use std::fs::File;

use backreader::BackBufRead;
use backreader::BackBufReader;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Error argument:\n  {} <file-path>", args[0]);
    }

    let f = File::open(args[1].clone()).unwrap();
    let bufreader = BackBufReader::new(f);
    bufreader
        .lines()
        .for_each(|line| println!("{}", line.unwrap()));
}
