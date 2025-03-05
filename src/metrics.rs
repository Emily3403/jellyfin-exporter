use chrono::{DateTime, Utc};
use prometheus_exporter::prometheus::{IntGauge, IntGaugeVec, register_int_gauge, register_int_gauge_vec};
use serde::{Deserialize, Serialize};

pub struct Metrics {
    pub jellyfin_up: IntGauge,
    pub jellyfin_config: IntGaugeVec,
    pub jellyfin_users: IntGaugeVec,
    pub jellyfin_sessions: IntGaugeVec,
    pub jellyfin_devices: IntGaugeVec,
    pub jellyfin_items_count: IntGaugeVec,

    pub jellyfin_items_library: IntGaugeVec,
    pub jellyfin_items_library_user_data: IntGaugeVec,
    pub jellyfin_items_media_item: IntGaugeVec,
    pub jellyfin_items_media_item_user_data: IntGaugeVec,
    pub jellyfin_items_season: IntGaugeVec,
    pub jellyfin_items_season_user_data: IntGaugeVec,
    pub jellyfin_items_episode: IntGaugeVec,
    pub jellyfin_items_episode_user_data: IntGaugeVec,
}

pub fn register_metrics() -> Metrics {
    Metrics {
        jellyfin_up: register_int_gauge!("jellyfin_up", "Indicates if the metrics could be scraped by the exporter.").unwrap(),
        jellyfin_config: JellyfinConfig::register(),
        jellyfin_users: User::register(),
        jellyfin_sessions: Session::register(),
        jellyfin_devices: Device::register(),
        jellyfin_items_count: ItemCounts::register(),
        jellyfin_items_library: Library::register(),
        jellyfin_items_library_user_data: UserData::register("library"),
        jellyfin_items_media_item: MediaItem::register(),
        jellyfin_items_media_item_user_data: UserData::register("media_item"),
        jellyfin_items_season: Season::register(),
        jellyfin_items_season_user_data: UserData::register("season"),
        jellyfin_items_episode: Episode::register(),
        jellyfin_items_episode_user_data: UserData::register("episode"),
    }
}

pub fn set_jellyfin_up(metrics: &mut Metrics) {
    metrics.jellyfin_up.set(1);
}


// This is the most efficient way of handling labeling as the `.with` and `HashMap` variant has "a much higher overhead"
// The drawback, of course, is the ability to have an incorrect number of arguments or have them in the wrong order.
// To avoid this, we tightly couple these two implementations and check in tests if the order and cardinality are correct.
pub trait ExportableMetric {
    fn set_metrics(&self, metrics: &mut Metrics);
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub name: String,
    pub id:   String,

    // TODO: These are 60min wrong due to Jellyfin reporting as GMT. I'm not quite sure how to fix that yet.
    pub last_login_date:    DateTime<Utc>,
    pub last_activity_date: DateTime<Utc>,
    // TODO: other fields. I think only policy.is_administrator could be interesting
}

impl User {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_users", "The registered Jellyfin users", &["name", "id", "last_login_date", "last_activity_date"]).unwrap()
    }
}


impl ExportableMetric for User {
    fn set_metrics(&self, metrics: &mut Metrics) {
        metrics.jellyfin_users.with_label_values(&[&self.name, &self.id, &self.last_activity_date.to_string(), &self.last_activity_date.to_string()]).set(1);
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Device {
    name: String,
    id: String,
    last_user_name: String,
    last_user_id: String,

    app_name: String,
    app_version: String,
    date_last_activity: DateTime<Utc>,
}

impl Device {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_devices", "The devices of this Jellyfin instance", &["name", "id", "last_user_name", "last_user_id", "app_name", "app_version", "date_last_activity"])
            .unwrap()
    }
}


impl ExportableMetric for Device {
    fn set_metrics(&self, metrics: &mut Metrics) {
        metrics.jellyfin_devices.with_label_values(&[&self.name, &self.id, &self.last_user_name, &self.last_user_id, &self.app_name, &self.app_version, &self.date_last_activity.to_string()]).set(1);
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    id: String,
    user_id: String,
    user_name: String,
    server_id: String,
    is_active: bool,

    client: String,
    device_name: String,
    device_id: String,
    application_version: String,

    remote_end_point:   String,
    last_activity_date: DateTime<Utc>,

    play_state: PlayState,
    now_playing_item: Option<Item>,
    transcoding_info: Option<TranscodingInfo>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PlayState {
    position_ticks: Option<i64>,
    play_method: Option<String>,
    is_paused: bool,
    is_muted: bool,
    volume_level: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TranscodingInfo {
    audio_codec: String,
    video_codec: String,
    container: String,
    is_video_direct: bool,
    is_audio_direct: bool,
    bitrate: i32,
    framerate: Option<i32>,
    completion_percentage: Option<f64>,
    width: i32,
    height: i32,
    hardware_acceleration_type: Option<String>,
}


impl Session {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_sessions", "The sessions of this Jellyfin instance", &[
            "id", "user_id", "user_name", "server_id", "is_active", "client", "device_name", "device_id", "application_version", "remote_end_point", "last_activity_date"
        ])
        .unwrap()
    }
}


impl ExportableMetric for Session {
    fn set_metrics(&self, metrics: &mut Metrics) {
        metrics
            .jellyfin_sessions
            .with_label_values(&[
                &self.id,
                &self.user_id,
                &self.user_name,
                &self.server_id,
                &self.is_active.to_string(),
                &self.client,
                &self.device_name,
                &self.device_id,
                &self.application_version,
                &self.remote_end_point,
                &self.last_activity_date.to_string(),
            ])
            .set(1);
        println!("{:?}", self);
    }
}


#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct JellyfinConfig {
    pub local_address: String,
    pub server_name: String,
    pub version: String,
    pub id: String,
}

impl JellyfinConfig {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_config", "The configuration of this Jellyfin instance", &["local_address", "server_name", "version", "id"]).unwrap()
    }
}

impl ExportableMetric for JellyfinConfig {
    fn set_metrics(&self, metrics: &mut Metrics) {
        metrics.jellyfin_users.with_label_values(&[&self.local_address, &self.server_name, &self.version, &self.id]).set(1);
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ItemCounts {
    pub movie_count: Option<i64>,
    pub series_count: Option<i64>,
    pub episode_count: Option<i64>,
    pub artist_count: Option<i64>,
    pub program_count: Option<i64>,
    pub trailer_count: Option<i64>,
    pub song_count: Option<i64>,
    pub album_count: Option<i64>,
    pub music_video_count: Option<i64>,
    pub box_set_count: Option<i64>,
    pub book_count: Option<i64>,
    pub item_count: Option<i64>,
}

impl ItemCounts {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_item_counts", "The item counts of this Jellyfin instance", &["type"]).unwrap()
    }
}


impl ExportableMetric for ItemCounts {
    fn set_metrics(&self, metrics: &mut Metrics) {
        metrics.jellyfin_items_count.with_label_values(&["Movie"]).set(self.movie_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Series"]).set(self.series_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Episode"]).set(self.episode_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Artist"]).set(self.artist_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Program"]).set(self.program_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Trailer"]).set(self.trailer_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Song"]).set(self.song_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Album"]).set(self.album_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Music Video"]).set(self.music_video_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Box Set"]).set(self.box_set_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Book"]).set(self.book_count.unwrap_or(0));
        metrics.jellyfin_items_count.with_label_values(&["Item"]).set(self.item_count.unwrap_or(0));
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "Type")]
pub enum Item {
    CollectionFolder(Library),
    Series(MediaItem),
    Movie(MediaItem),
    Book(MediaItem),

    Season(Season),
    Episode(Episode),

    // Not used
    Folder,
    ManualPlaylistsFolder,
}

impl Item {
    pub fn set_metrics(&self, metrics: &mut Metrics, user: &User) {
        match self {
            Item::CollectionFolder(it) => it.set_metrics(metrics, user),
            Item::Series(it) => it.set_metrics(metrics, user),
            Item::Movie(it) => it.set_metrics(metrics, user),
            Item::Book(it) => it.set_metrics(metrics, user),
            Item::Season(it) => it.set_metrics(metrics, user),
            Item::Episode(it) => it.set_metrics(metrics, user),
            Item::Folder => {}
            Item::ManualPlaylistsFolder => {}
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Library {
    pub name: String,
    pub server_id: String,
    pub id: String,
    pub collection_type: String, // TODO: Make this a better value, currently it is "movies", "tvshows", etc.

    pub user_data: Option<UserData>, // TODO: I would like to have the physical paths attached to this library
}


impl Library {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_items_library", "The available Jellyfin libraries", &["user_name", "user_id", "name", "server_id", "id", "collection_type"]).unwrap()
    }
}

macro_rules! to_nullable_string {
    ($it:expr) => {
        &$it.map(|it| it.to_string()).unwrap_or("null".to_string())
    };
}

impl Library {
    pub fn set_metrics(&self, metrics: &mut Metrics, user: &User) {
        metrics.jellyfin_items_library.with_label_values(&[&user.name, &user.id, &self.name, &self.server_id, &self.id, &self.collection_type]).set(1);
        if let Some(it) = &self.user_data {
            it.set_metrics(metrics, user)
        };
    }
}


#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MediaItem {
    pub name: String,
    pub server_id: String,
    pub id: String,

    pub location_type: String,
    pub media_type:    String,

    pub premiere_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub production_year: Option<i32>,
    pub official_rating: Option<String>,
    pub community_rating: Option<f64>,
    pub status: Option<String>,

    pub user_data: Option<UserData>,
}

impl MediaItem {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_items_media_item", "The available Jellyfin MediaItems (Series, Movies, Books)", &[
            "user_name", "user_id", "name", "server_id", "id", "location_type", "media_type", "premiere_date", "end_date", "production_year", "official_rating", "community_rating", "status"
        ])
        .unwrap()
    }

    pub fn set_metrics(&self, metrics: &mut Metrics, user: &User) {
        metrics
            .jellyfin_items_media_item
            .with_label_values(&[
                &user.name,
                &user.id,
                &self.name,
                &self.server_id,
                &self.id,
                &self.location_type,
                &self.media_type,
                to_nullable_string!(self.premiere_date),
                to_nullable_string!(self.end_date),
                to_nullable_string!(self.production_year),
                to_nullable_string!(self.official_rating.clone()),
                to_nullable_string!(self.community_rating),
                to_nullable_string!(self.status.clone()),
            ])
            .set(1);
        if let Some(it) = &self.user_data {
            it.set_metrics(metrics, user)
        };
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Season {
    pub series_name: String,
    pub series_id:   String,

    pub name: String,
    pub server_id: String,
    pub id: String,

    pub index_number:    Option<i32>,
    pub premiere_date:   Option<DateTime<Utc>>, // Each season has its own premiere_date
    pub production_year: Option<i32>,

    pub user_data: Option<UserData>,
}

impl Season {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_items_season", "The available Jellyfin seasons", &["user_name", "user_id", "name", "server_id", "id", "index_number", "premiere_date", "production_year"])
            .unwrap()
    }

    pub fn set_metrics(&self, metrics: &mut Metrics, user: &User) {
        metrics
            .jellyfin_items_season
            .with_label_values(&[
                &user.name,
                &user.id,
                &self.name,
                &self.server_id,
                &self.id,
                to_nullable_string!(self.index_number),
                to_nullable_string!(self.premiere_date),
                to_nullable_string!(self.production_year),
            ])
            .set(1);
        if let Some(it) = &self.user_data {
            it.set_metrics(metrics, user)
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Episode {
    pub series_name: Option<String>,
    pub series_id:   Option<String>,

    pub season_name: Option<String>,
    pub season_id: Option<String>,
    pub parent_index_number: Option<i32>,

    pub name: String,
    pub server_id: String,
    pub id: String,

    pub has_subtitles: Option<bool>,
    pub container: Option<String>,
    pub path: Option<String>,
    pub run_time_ticks: Option<i64>,

    pub index_number:    Option<i32>,
    pub premiere_date:   Option<DateTime<Utc>>, // Each season has its own premiere_date
    pub production_year: Option<i32>,

    user_data: Option<UserData>,
}

impl Episode {
    pub fn register() -> IntGaugeVec {
        register_int_gauge_vec!("jellyfin_items_episode", "The available Jellyfin episodes", &[
            "user_name", "user_id", "name", "server_id", "id", "has_subtitles", "container", "path", "index_number", "premiere_date", "production_year"
        ])
        .unwrap()
    }

    pub fn set_metrics(&self, metrics: &mut Metrics, user: &User) {
        metrics
            .jellyfin_items_episode
            .with_label_values(&[
                &user.name,
                &user.id,
                &self.name,
                &self.server_id,
                &self.id,
                to_nullable_string!(self.has_subtitles),
                to_nullable_string!(self.container.clone()),
                to_nullable_string!(self.path.clone()),
                to_nullable_string!(self.index_number),
                to_nullable_string!(self.premiere_date),
                to_nullable_string!(self.production_year),
            ])
            .set(1);
        if let Some(it) = &self.user_data {
            it.set_metrics(metrics, user)
        };
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ManualPlaylists {}


#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct UserData {
    pub play_count:  i32, // Series never have a play count, only episodes do.
    pub played:      bool,
    pub is_favorite: bool,

    pub unplayed_item_count: Option<i32>,
    pub last_played_date:    Option<DateTime<Utc>>,
    pub played_percentage:   Option<f64>,
}

impl UserData {
    pub fn register(name: &str) -> IntGaugeVec {
        register_int_gauge_vec!(&format!("jellyfin_items_{}_user_data", name), "User Data of Libraries", &[
            "user_name", "user_id", "play_count", "played", "is_favorite", "unplayed_item_count", "last_played_date", "played_percentage"
        ])
        .unwrap()
    }

    pub fn set_metrics(&self, metrics: &mut Metrics, user: &User) {
        metrics
            .jellyfin_items_library_user_data
            .with_label_values(&[
                &user.name,
                &user.id,
                &self.play_count.to_string(),
                &self.played.to_string(),
                &self.is_favorite.to_string(),
                to_nullable_string!(self.unplayed_item_count),
                to_nullable_string!(self.last_played_date),
                to_nullable_string!(self.played_percentage),
            ])
            .set(1);
    }
}
