use clap::Parser;
use webhook_cli::{
    error::Error,
    utils::{Cli, Configs, ConfigsBuilder, ConfigsEnvironment},
};

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let configs_from_env = ConfigsEnvironment::new();

    if let Err(e) = configs_from_env {
        eprintln!("The program encountered an error: {}", e);
        std::process::exit(1);
    }

    let configs: Configs = ConfigsBuilder::new()
        .unwrap()
        .ssl_verify(false)
        .from_env(configs_from_env.unwrap())
        .build();

    if let Err(e) = args.run(configs) {
        eprintln!("The program encountered an error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
