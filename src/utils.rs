use crate::error::Error;
use anyhow::anyhow;
use anyhow::Context;
use clap::{ArgAction, Parser};
use config;
use reqwest;
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct Configs {
    inventory_path: String,
    user_agent: String,
    ssl_verify: bool,
}

#[derive(Debug)]
pub struct ConfigsBuilder {
    inventory_path: String,
    user_agent: String,
    ssl_verify: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConfigsEnvironment {
    inventory_path: Option<String>,
    user_agent: Option<String>,
    ssl_verify: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
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

    /// Do not send the actual request
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub simulate: bool,

    /// URL value, replaces $URL on the template
    #[arg(short, action = ArgAction::Set, value_name="LINK")]
    pub url: Option<String>,

    /// Value to inject, starting from $1...$n
    #[arg(short, action = ArgAction::Append, value_name="VALUE")]
    pub inject: Option<Vec<String>>,

    /// Enables enhanced logging
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub verbose: bool,
}

impl Cli {
    pub fn run(&self, c: Configs) -> Result<(), Error> {
        if self.list {
            if self.webhook.is_some() || self.inject.is_some() {
                return Err(Error::InvalidArgsError);
            }

            self.list_hooks(&c)?;
        }

        if let Some(webhook) = &self.webhook {
            let req = self.build_webhook_request(webhook.to_string(), c)?.unwrap();

            let res = req.send().context("request failed")?;

            // TODO add verbosity levels
            if res.status() == reqwest::StatusCode::OK {
                println!("Response OK");
                if self.verbose {
                    println!("{}", res.text().context("failed extracting reponse text")?);
                }
            } else {
                eprintln!("Response ERR");
                if self.verbose {
                    eprintln!("{}", res.text().context("failed extracting reponse text")?);
                }
            }
        }

        Ok(())
    }

    fn build_webhook_request(
        &self,
        webhook: String,
        c: Configs,
    ) -> Result<Option<RequestBuilder>, Error> {
        let mut filename = webhook;
        if filename.ends_with(".json") {
            filename = String::from(
                filename
                    .strip_suffix(".json")
                    .context("error while stripping suffix")?,
            );
        }

        let mut content = fs::read_to_string(format!("{}/{}.json", c.inventory_path, filename))
            .map_err(|e| anyhow!(e))?;

        // URL injection
        if let Some(u) = &self.url {
            content = content.replace("$URL", u);
        }

        // Inject $1..n values
        if let Some(vec) = &self.inject {
            if self.verbose {
                println!("Injecting values into template: {:?}", vec);
            }

            vec.iter().enumerate().for_each(|(i, x)| {
                content = content.replace(format!("${}", i + 1).as_str(), x);
            });
        }

        let json_content: WebHookTemplate =
            serde_json::from_str(&content).map_err(|e| anyhow!(e))?;

        // Builds the request
        let req = self.build_request(json_content.clone(), &c)?;

        if self.verbose {
            println!("Template content: {:?}", json_content);
        }

        if self.simulate {
            return Ok(None);
        }

        Ok(Some(req))
    }

    fn list_hooks(&self, c: &Configs) -> Result<(), Error> {
        let paths: Vec<PathBuf> = fs::read_dir(&c.inventory_path)
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
            if let Some(stem) = path.file_stem() {
                println!("- {}", stem.to_str().unwrap_or_default());
            }
        }

        Ok(())
    }

    pub fn build_request(&self, w: WebHookTemplate, c: &Configs) -> Result<RequestBuilder, Error> {
        let client = Client::builder()
            .user_agent(&c.user_agent)
            .danger_accept_invalid_certs(!c.ssl_verify)
            .build()
            .context("couldn't build the request")?;

        let body = serde_json::to_string(&w.data).context("error while serializing")?;

        let res = client
            .post(reqwest::Url::parse(w.url.as_str()).context("couldn't parse URL")?)
            .body(body);

        return Ok(res);
    }
}

impl Default for ConfigsBuilder {
    fn default() -> Self {
        Self {
            inventory_path: "./inventory/".to_string(),
            user_agent: concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).to_string(),
            ssl_verify: true,
        }
    }
}
impl ConfigsBuilder {
    pub fn new() -> Result<ConfigsBuilder, Error> {
        let c = ConfigsBuilder::default();
        Ok(c)
    }

    pub fn from_env(mut self, c: ConfigsEnvironment) -> ConfigsBuilder {
        if let Some(inv) = c.inventory_path {
            self = self.inventory_path(inv);
        }
        self
    }

    pub fn inventory_path(mut self, path: String) -> ConfigsBuilder {
        self.inventory_path = path;
        self
    }
    pub fn user_agent(mut self, user_agent: String) -> ConfigsBuilder {
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

impl ConfigsEnvironment {
    pub fn new() -> Result<ConfigsEnvironment, Error> {
        let mut env_configs = config::Config::builder();
        env_configs =
            env_configs.add_source(config::Environment::default().prefix("APP").separator("__"));

        let from_env_configs: ConfigsEnvironment = env_configs
            .build()
            .context("couldn't build configs from environment")?
            .try_deserialize::<ConfigsEnvironment>()
            .map_err(|e| anyhow!(e))?;

        Ok(from_env_configs)
    }
}
