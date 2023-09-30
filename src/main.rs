use anyhow::anyhow;
use clap::{Parser, ArgAction};
use serde_json::Value;
use std::fs;
use webhook_cli::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Webhook to execute
    webhook: Option<String>,

    /// Lists all available webhooks
    #[arg(short, long, action =ArgAction::SetTrue)]
    list: bool,
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    if args.list == true && args.webhook.is_some() {
        Err(Error::InvalidArgsError)?
    }

    if let Some(wbh) = &args.webhook {
        let content =
            fs::read_to_string(format!("./inventory/{}.json", wbh)).map_err(|e| anyhow!(e))?;
        let payload_data: Value = serde_json::from_str(&content).map_err(|e| anyhow!(e))?;
        println!("{:?}", payload_data);
    }
    Ok(())
}
