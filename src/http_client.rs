use crate::api::{get_devices, get_item_counts, get_items, get_jellyfin_config, get_jellyfin_up, get_sessions, get_users, validate_items};
use crate::cli::Cli;
use crate::metrics::{Device, ExportableMetric, Metrics, set_config_metrics, set_device_metrics, set_item_count_metrics, set_item_metrics, set_jellyfin_up, set_session_metrics, set_user_metrics};
use log::{debug, error, warn};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Instant;

// For now, we will have blocking calls to the API, as this is the simplest way.
// But in the future, I want to handle this with async and tokio, as parallel querying and processing of the API is essential for a responsive exporter
pub fn client(cli: &Cli) -> Client {
    Client::builder().default_headers(headers(cli)).build().expect("Building the HTTP Client failed")
}

pub fn headers(cli: &Cli) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let auth_header = HeaderValue::from_str(&format!(r#"MediaBrowser Token="{}", Client="Jellyfin-Exporter", Version={}"#, cli.jellyfin_api_key, env!("CARGO_PKG_VERSION"))).unwrap();
    headers.append("Authorization", auth_header);
    headers.append("Content-Type", HeaderValue::from_str("application/json").unwrap());

    headers
}

macro_rules! fatal_error {
    ($it:expr, $err_text:expr) => {
        $it.map_err(|e| {
            error!($err_text);
            e
        })?
    };
}

macro_rules! log_error {
    ($it:expr, $err_text:expr) => {
        $it.map_err(|e| {
            warn!(concat!($err_text, ": {:?}"), e);
            e
        })
    };
}


pub async fn handle_request(cli: &Cli, client: &Client, metrics: &mut Metrics) -> Result<(), reqwest::Error> {
    let (is_up, config, users, devices, item_counts, sessions) =
        tokio::join!(get_jellyfin_up(cli, client), get_jellyfin_config(cli, client), get_users(cli, client), get_devices(cli, client), get_item_counts(cli, client), get_sessions(cli, client));
    fatal_error!(is_up, "Jellyfin Server is down!");
    set_jellyfin_up(metrics);

    if let Ok(config) = log_error!(config, "Could not get Jellyfin Info") {
        set_config_metrics(&config, metrics)
    };

    if let Ok(sessions) = log_error!(sessions, "Could not get Sessions") {
        set_session_metrics(&sessions, metrics)
    }

    if let Ok(devices) = log_error!(devices, "Could not get Devices") {
        set_device_metrics(&devices, metrics)
    }

    if let Ok(item_counts) = log_error!(item_counts, "Could not get Item Counts") {
        set_item_count_metrics(&item_counts, metrics)
    }

    if let Ok(users) = log_error!(users, "Could not get Users") {
        set_user_metrics(&users, metrics);
        let items = get_items(cli, client, users, metrics).await;

        for (user, items) in items {
            validate_items(&items);
            set_item_metrics(&items, metrics, &user)
        }
    };


    Ok(())
}
