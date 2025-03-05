use clap::Parser;
use std::env;
use std::fmt::{Display, Formatter, write};
use std::net::IpAddr;
use url::{ParseError, Url};

// TODO: Add option to not parse library content as this can very CPU / network intensive
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, env, default_value = "0.0.0.0")]
    pub jellyfin_exporter_address: IpAddr,

    #[arg(long, env, default_value = "9741")]
    pub jellyfin_exporter_port: u16,

    #[arg(long, env, default_value = "info", value_parser=parse_loglevel)]
    pub jellyfin_exporter_loglevel: String,

    #[arg(long, env)]
    pub jellyfin_exporter_insecure: bool,

    #[arg(long, env, value_parser=parse_url)]
    pub jellyfin_address: Url,

    #[arg(long, env)]
    pub jellyfin_api_key: String,

    #[arg(long, env, default_value = "false", help = "Disable recursive item search. Helps to decrease CPU / Memory usage as this results in expensive API calls")]
    pub jellyfin_exporter_disable_recursive_item_search: bool,
}

pub fn parse_url(url: &str) -> Result<Url, String> {
    let url = match Url::parse(url) {
        Ok(it) => it,
        Err(ParseError::RelativeUrlWithoutBase) => Err("Missing scheme: Add http(s):// in front of the URL")?,
        Err(it) => Err(it.to_string())?,
    };

    // We can't use the clap config object to evaluate the insecure flag, so do it manually
    let is_insecure = env::args().any(|arg| arg == "--insecure") || env::var("INSECURE").is_ok();
    let scheme = url.scheme();

    if !is_insecure && scheme == "http" {
        let ip_addrs = match url.socket_addrs(|| None) {
            Ok(it) => it,
            Err(it) => Err(it.to_string())?,
        };

        if ip_addrs.into_iter().any(|it| !it.ip().is_loopback()) {
            return Err("Insecure connection detected: http with a non-loopback (localhost) address! Aborting to not send the API Key via plain text!\n\nTo override this behaviour specify the --insecure flag".into());
        }
    };

    if scheme != "http" && scheme != "https" {
        return Err(format!("Invalid scheme: \"{scheme}\""));
    }

    Ok(url)
}

pub fn parse_loglevel(level: &str) -> Result<String, String> {
    // log::LOG_LEVEL_NAMES is private :/
    let level = level.to_lowercase();
    if level == "off" || level == "error" || level == "warn" || level == "info" || level == "debug" || level == "trace" {
        unsafe { env::set_var("RUST_LOG", &level) };
        return Ok(level.into());
    }

    Err(format!(r#"Expected loglevel to be in {{"off", "error", "warn", "info", "debug", "trace"}}, got "{level}""#))
}

impl Display for Cli {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"Cli {{
    jellyfin_exporter_address  = {}
    jellyfin_exporter_port     = {}
    jellyfin_exporter_loglevel = {}
    jellyfin_exporter_insecure = {}

    jellyfin_address           = {}
    jellyfin_api_key           = <REDACTED>

    disable_recursive_item_search = {}
}}"#,
            self.jellyfin_exporter_address,
            self.jellyfin_exporter_port,
            self.jellyfin_exporter_loglevel,
            self.jellyfin_exporter_insecure,
            self.jellyfin_address.to_string(),
            self.jellyfin_exporter_disable_recursive_item_search,
        )
    }
}
