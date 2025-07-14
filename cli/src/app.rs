use std::process::exit;

use crate::{
    args::Args,
    config::{Config, Host},
};

#[derive(Debug)]
pub struct App {
    pub args: Args,
    pub config: Config,
}

impl App {
    pub fn get_host(&self) -> Host {
        if self.config.default_host.is_none() && self.args.host.is_none() {
            tracing::error!(
                "No host was provided, use --host <url>, add a new host via auth add <options> or set an already existing host as the default with auth set <name>"
            );
            exit(0);
        }

        if let Some(host_str) = &self.args.host {
            match host_str.split_once('@') {
                Some((token, url)) => Host {
                    name: String::from("ArgsHost"),
                    url: url.to_string(),
                    token: Some(token.to_string()),
                    web_url: None,
                },
                None => Host {
                    name: String::from("ArgsHost"),
                    url: host_str.clone(),
                    token: None,
                    web_url: None,
                },
            }
        } else {
            if let Some(default_host) = &self.config.default_host {
                self.config
                    .hosts
                    .iter()
                    .filter(|h| &h.name == default_host)
                    .collect::<Vec<&Host>>()
                    .first()
                    .cloned()
                    .unwrap()
                    .clone()
            } else {
                panic!("No valid host found")
            }
        }
    }

    pub fn get_base_socket_url(&self) -> String {
        let host = self.get_host();
        let secure = host.url.starts_with("https");
        let domain = host
            .url
            .strip_prefix(if secure { "https" } else { "http" })
            .expect("Invalid url, doesn't start with https or http");

        format!("{}{domain}", if secure { "wss" } else { "ws" })
    }

    pub fn add_auth_to_req<T>(&self, req: ureq::RequestBuilder<T>) -> ureq::RequestBuilder<T> {
        let host = self.get_host();
        if let Some(token) = host.token {
            req.header("Authorization", format!("Bearer {token}"))
        } else {
            req
        }
    }

    pub fn add_agent_to_req<T>(&self, req: ureq::RequestBuilder<T>) -> ureq::RequestBuilder<T> {
        req.header("User-Agent", App::get_user_agent())
    }

    pub fn get_user_agent() -> String {
        format!("sf_cli / {}", env!("CARGO_PKG_VERSION"))
    }

    pub fn get_url(&self, path: impl AsRef<str>) -> String {
        let host = self.get_host();
        format!("{}{}", host.url, path.as_ref())
    }
}
