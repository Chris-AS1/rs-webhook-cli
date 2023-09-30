use anyhow::anyhow;
use anyhow::Context;
use clap::{ArgAction, Parser};
use reqwest;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::{fs, path::PathBuf};
use webhook_cli::error::Error;

#[derive(Deserialize, Debug, Serialize)]
struct WebHookTemplate {
    url: DestURL,
    data: ReqwPayload,
}

#[derive(Deserialize, Debug, Serialize)]
struct DestURL(String);

#[derive(Deserialize, Debug, Serialize)]
struct ReqwPayload(Value);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Webhook to execute
    webhook: Option<String>,

    /// Lists all available webhooks
    #[arg(short, long, action = ArgAction::SetTrue)]
    list: bool,

    /// Value to inject, will be treated in order
    #[arg(short, action = ArgAction::Append, value_name="VALUE")]
    inject: Option<Vec<String>>,
}

fn list_hooks() -> Result<(), Error> {
    let paths: Vec<PathBuf> = fs::read_dir("./inventory/")
        .map_err(|e| anyhow!(e))?
        .filter_map(|f| f.ok())
        .filter(|f| match f.path().extension() {
            Some(ex) => ex == "json",
            None => false,
        })
        .map(|f| f.path())
        .collect();

    for path in paths {
        println!("{}", path.display());
    }

    Ok(())
}

fn build_request(w: WebHookTemplate) -> Result<(), Error> {
    let client = Client::new();
    let body = serde_json::to_string(&w.data);

    let res = client
        .post(reqwest::Url::parse(w.url.0.as_str()).context("url not valid")?)
        .body(body.context("something went wrong")?.clone());

    println!("{:?}", res);

    res.send().context("request failed")?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    if args.list == true {
        if args.webhook.is_some() || args.inject.is_some() {
            Err(Error::InvalidArgsError)?
        }
        match list_hooks() {
            Ok(_) => true,
            Err(e) => Err(e)?,
        };
    };

    if let Some(wbh) = &args.webhook {
        let mut wbh_filename = String::from(wbh);
        if wbh.ends_with(".json") {
            let a = wbh.strip_suffix(".json").unwrap();
            wbh_filename = a.to_string();
        }

        let content = fs::read_to_string(format!("./inventory/{}.json", wbh_filename))
            .map_err(|e| anyhow!(e))?;
        let payload_data: WebHookTemplate =
            serde_json::from_str(&content).map_err(|e| anyhow!(e))?;

        build_request(payload_data).map_err(|e| anyhow!(e))?
    }

    if let Some(vec) = &args.inject {
        println!("Injecting values: {:?}", vec);
    }

    Ok(())
}
