use std::env;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    webhook: String,
}

fn main() {
    let args = Cli::parse();
}
