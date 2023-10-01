use clap::Parser;
use webhook_cli::{
    error::Error,
    utils::{Cli, Configs, ConfigsBuilder},
};

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let configs: Configs = ConfigsBuilder::default().ssl_verify(false).build();

    if let Err(e) = args.run(configs) {
        eprintln!("The program encountered an error: {}", e);
    }

    Ok(())
}
