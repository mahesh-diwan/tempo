//! Anonymous YouTube Music search.
//!
//! v1 scope: search songs with the unauthenticated InnerTube metadata client.
//! No account, cookie, cipher, PoToken, or `WEB_REMIX` stream path is used.

use tempo_model::SongItem;

use crate::{Clients, InnerTube, METADATA_CLIENT};

#[derive(Debug, thiserror::Error)]
pub enum YoutubeError {
    #[error("search failed: {0}")]
    Request(String),
    #[error("metadata client {0} is missing from the registry")]
    MissingClient(String),
}

/// Search songs for `query`. An empty result is `Ok(vec![])`, not an error.
pub async fn search_songs(
    it: &InnerTube,
    query: &str,
) -> Result<Vec<SongItem>, YoutubeError> {
    let clients = Clients::bundled();
    let client = clients
        .get(METADATA_CLIENT)
        .ok_or_else(|| YoutubeError::MissingClient(METADATA_CLIENT.to_owned()))?;
    let result = it
        .search_songs(client, query)
        .await
        .map_err(|e| YoutubeError::Request(e.to_string()))?;
    Ok(result.items.into_iter().map(map_song).collect())
}

/// Map the crate's parsed song row onto the shared model type.
pub fn map_song(item: crate::models::metadata::SongItem) -> SongItem {
    SongItem {
        video_id: item.video_id,
        title: item.title,
        artists: item.artists,
        album: item.album,
        duration: item.duration,
        thumbnail: item.thumbnail,
        is_video: item.is_video,
    }
}

/// Parse a raw search response into shared song items.
pub fn parse_song_items(root: &serde_json::Value) -> Vec<SongItem> {
    crate::models::metadata::parse_search(root)
        .items
        .into_iter()
        .map(map_song)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_a_song_row() {
        let row = json!({
            "musicResponsiveListItemRenderer": {
                "playlistItemData": { "videoId": "vid1" },
                "flexColumns": [
                    { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                        { "text": "Song" }
                    ] } } },
                    { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                        { "text": "Adele" }, { "text": " • " },
                        { "text": "Album" }, { "text": " • " },
                        { "text": "3:45" }
                    ] } } }
                ],
                "fixedColumns": [
                    { "musicResponsiveListItemFixedColumnRenderer": { "text": { "runs": [
                        { "text": "3:45" }
                    ] } } }
                ]
            }
        });
        let items = parse_song_items(&row);
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.video_id, "vid1");
        assert_eq!(item.title, "Song");
        assert_eq!(item.artists, "Adele");
        assert_eq!(item.album.as_deref(), Some("Album"));
        assert_eq!(item.duration.as_deref(), Some("3:45"));
        assert!(!item.is_video);
    }

    #[test]
    fn empty_response_yields_no_items_and_no_error() {
        assert!(parse_song_items(&json!({})).is_empty());
    }

    #[test]
    fn non_ascii_search_query_round_trips() {
        let query = "अरिजीत सिंह";
        let encoded = urlencoding::encode(query);
        assert!(!encoded.is_empty());
        assert_eq!(encoded, "%E0%A4%85%E0%A4%B0%E0%A4%BF%E0%A4%9C%E0%A5%80%E0%A4%A4%20%E0%A4%B8%E0%A4%BF%E0%A4%82%E0%A4%B9");
    }
}
