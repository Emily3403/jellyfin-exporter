use crate::cli::Cli;
use crate::metrics::{Device, Item, ItemCounts, JellyfinConfig, Metrics, Session, User};
use futures::StreamExt;
use log::warn;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
// TODO:
// - Sessions
//   - Active / Total
//   - NowPlayingItem
// - Tickplay
// - How much GPU / transcoding power is used currently

// Should I have a server_id in every metric?


// Exporting:
// I want num_items per library, associated with user
// → each exported item has to be associated with the library and user
// How much total runtime each show has, how many episodes etc

pub async fn make_api_get_call(cli: &Cli, client: &Client, path: &str) -> Result<Response, reqwest::Error> {
    client.get(cli.jellyfin_address.clone().join(path).unwrap()).send().await
}

pub async fn make_api_post_call(cli: &Cli, client: &Client, path: &str) -> Result<Response, reqwest::Error> {
    client.post(cli.jellyfin_address.clone().join(path).unwrap()).send().await
}

pub async fn get_users(cli: &Cli, client: &Client) -> Result<Vec<User>, reqwest::Error> {
    make_api_get_call(cli, client, "/Users").await?.json().await
}

pub async fn get_sessions(cli: &Cli, client: &Client) -> Result<Vec<Session>, reqwest::Error> {
    make_api_get_call(cli, client, "/Sessions").await?.json().await
}

pub async fn get_jellyfin_config(cli: &Cli, client: &Client) -> Result<JellyfinConfig, reqwest::Error> {
    make_api_get_call(cli, client, "/System/Info").await?.json().await
}

pub async fn get_devices(cli: &Cli, client: &Client) -> Result<Vec<Device>, reqwest::Error> {
    Ok(make_api_get_call(cli, client, "/Devices").await?.json::<ItemResponse<Device>>().await?.items)
}

pub async fn get_item_counts(cli: &Cli, client: &Client) -> Result<ItemCounts, reqwest::Error> {
    make_api_get_call(cli, client, "/Items/Counts").await?.json().await
}

pub async fn get_jellyfin_up(cli: &Cli, client: &Client) -> Result<String, reqwest::Error> {
    make_api_post_call(cli, client, "/System/Ping").await?.json().await
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
struct ItemResponse<T> {
    items: Vec<T>,
    total_record_count: i32,
    start_index: i32,
}


/// Querying on a per-user basis is done because of two reasons:
/// 1. Libraries (`CollectionFolder`) are not returned when only requesting from `/Items?recursive=true`.
/// 2. Performance: Jellyfin is able to parallelize multiple API requests.
///
/// TODO: Optimize the memory layout: currently megabytes of memory are allocated and thrown away
pub async fn get_items<'a>(cli: &Cli, client: &Client, users: Vec<User>, metrics: &mut Metrics) -> Vec<(User, Vec<Item>)> {
    futures::stream::iter(users.into_iter().map(|user| async move {
        let response = match make_api_get_call(cli, client, &format!("/Items?UserId={}&recursive={}", user.id, !cli.jellyfin_exporter_disable_recursive_item_search)).await {
            Ok(it) => it,
            Err(e) => {
                warn!("Could not fetch user {}: {:?}", user.name, e);
                return (user, Vec::new());
            }
        };

        match response.json::<ItemResponse<Item>>().await {
            Ok(it) => (user, it.items),
            Err(e) => {
                warn!("Could not decode item data for user {}: {:?}", user.name, e);
                return (user, Vec::new());
            }
        }
    }))
    .buffer_unordered(5)
    .collect::<Vec<_>>()
    .await
}

// TODO
pub fn validate_items(items: &Vec<Item>) -> bool {
    for item in items {
        match item {
            Item::CollectionFolder(it) => {}
            Item::Series(it) => {}
            Item::Movie(it) => {}
            Item::Book(it) => {}
            Item::Season(it) => {}
            Item::Episode(it) => {
                if it.series_name.is_none() || it.series_id.is_none() {
                    warn!("The episode \"{}\" ({}) does not have a series attached - this is probably a movie!", it.name, it.id);
                    return false;
                }
            }
            _ => {}
        }
    }

    true
}
