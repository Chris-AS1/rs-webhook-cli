use crate::error::Error;
use anyhow::anyhow;
use anyhow::Context;
use clap::{ArgAction, Parser};
use reqwest;
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct Configs {
    inventory_path: &'static str,
    user_agent: &'static str,
    ssl_verify: bool,
}

#[derive(Debug)]
pub struct ConfigsBuilder {
    inventory_path: &'static str,
    user_agent: &'static str,
    ssl_verify: bool,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WebHookTemplate {
    url: String,
    data: Value,
}

#[derive(Deserialize, Debug, Serialize)]
struct ReqwPayload(Value);

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
pub struct Cli {
    /// Webhook to execute
    pub webhook: Option<String>,

    /// Lists all available webhooks
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub list: bool,

    /// Value to inject, will be treated in order
    #[arg(short, action = ArgAction::Append, value_name="VALUE")]
    pub inject: Option<Vec<String>>,
}

impl Cli {
    pub fn run(&self, c: Configs) -> Result<(), Error> {
        if self.list {
            if self.webhook.is_some() || self.inject.is_some() {
                Err(Error::InvalidArgsError)?
            } else {
                match self.list_hooks(&c) {
                    Ok(_) => true,
                    Err(e) => Err(e)?,
                };
            };
        }

        if let Some(webhook) = &self.webhook {
            let mut filename = String::from(webhook);
            if filename.ends_with(".json") {
                filename = String::from(filename.strip_suffix(".json").unwrap());
            }

            let content = fs::read_to_string(format!("./inventory/{}.json", filename))
                .map_err(|e| anyhow!(e))?;

            let json_content: WebHookTemplate =
                serde_json::from_str(&content).map_err(|e| anyhow!(e))?;

            self.build_request(json_content, &c)?
                .send()
                .context("request failed")?;
        }

        if let Some(vec) = &self.inject {
            println!("Injecting values: {:?}", vec);
        }

        Ok(())
    }

    fn list_hooks(&self, c: &Configs) -> Result<(), Error> {
        let paths: Vec<PathBuf> = fs::read_dir(c.inventory_path)
            .map_err(|e| anyhow!(e))?
            .filter_map(|f| f.ok())
            .filter(|f| match f.path().extension() {
                Some(ex) => ex == "json",
                None => false,
            })
            .map(|f| f.path())
            .collect();

        match paths.len() {
            0 => println!("No webhooks were found."),
            _ => println!("The following webhooks were found:"),
        };

        for path in paths {
            println!("- {}", path.file_stem().unwrap().to_str().unwrap());
        }

        Ok(())
    }

    pub fn build_request(&self, w: WebHookTemplate, c: &Configs) -> Result<RequestBuilder, Error> {
        let client = Client::builder()
            .user_agent(c.user_agent)
            .danger_accept_invalid_certs(!c.ssl_verify)
            .build()
            .unwrap();

        let body = serde_json::to_string(&w.data).context("error while serializing")?;

        let res = client
            .post(reqwest::Url::parse(w.url.as_str()).context("couldn't parse URL")?)
            .body(body);

        println!("{:?}", res);
        return Ok(res);
    }
}

impl Default for ConfigsBuilder {
    fn default() -> Self {
        Self {
            inventory_path: "./inventory/",
            user_agent: concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            ssl_verify: true,
        }
    }
}

impl ConfigsBuilder {
    pub fn new() -> ConfigsBuilder {
        ConfigsBuilder::default()
    }

    pub fn inventory_path(mut self, path: &'static str) -> ConfigsBuilder {
        self.inventory_path = path;
        self
    }
    pub fn user_agent(mut self, user_agent: &'static str) -> ConfigsBuilder {
        self.user_agent = user_agent;
        self
    }

    pub fn ssl_verify(mut self, ssl_verify: bool) -> ConfigsBuilder {
        self.ssl_verify = ssl_verify;
        self
    }

    pub fn build(self) -> Configs {
        Configs {
            inventory_path: self.inventory_path,
            user_agent: self.user_agent,
            ssl_verify: self.ssl_verify,
        }
    }
}
