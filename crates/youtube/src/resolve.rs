//! Anonymous direct-URL stream resolution.
//!
//! Order: `VISIONOS`, `ANDROID_VR_1_65_10`, `ANDROID_VR_1_43_32`. `IOS` is excluded: it serves
//! only bounded-Range URLs and mpv never makes Range requests. Ciphered formats are skipped,
//! never deciphered. rustypipe is the last resort for a whole video id.

use std::collections::BTreeMap;
use std::future::Future;

use tempo_model::ResolvedStream;

use crate::{
    find_format, rustypipe_fallback, AudioQuality, Clients, InnerTube, PlayerResponse,
    STREAM_FALLBACK_ORDER,
};

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("no client could resolve a playable stream for {video_id}")]
    NoStream { video_id: String },
    #[error("stream request failed: {0}")]
    Request(String),
}

/// Resolve a video id using only the direct-URL clients. No network beyond the injected fetch.
///
/// `fetch` receives the registry key and video id and returns the raw `/player` JSON, or `None`
/// when the request failed.
pub async fn resolve_with<F, Fut>(
    clients: &Clients,
    fetch: F,
    video_id: &str,
    quality: AudioQuality,
) -> Result<ResolvedStream, ResolveError>
where
    F: Fn(&'static str, String) -> Fut,
    Fut: Future<Output = Option<PlayerResponse>>,
{
    for key in STREAM_FALLBACK_ORDER {
        let Some(client) = clients.get(key) else { continue };
        let Some(response) = fetch(key, video_id.to_owned()).await else {
            continue;
        };
        if !response.playability_status.is_ok() {
            continue;
        }
        let Some(streaming) = response.streaming_data.as_ref() else {
            continue;
        };
        let Some(expires) = streaming.expires_in_seconds else {
            continue;
        };
        let Some(format) = find_format(streaming, quality) else {
            continue;
        };
        // Ciphered formats are never deciphered in v1: skip and try the next client.
        let Some(url) = format.direct_url() else {
            continue;
        };
        let mut headers = BTreeMap::new();
        headers.insert("User-Agent".to_owned(), client.user_agent.clone());
        return Ok(ResolvedStream {
            video_id: video_id.to_owned(),
            url: url.to_owned(),
            headers,
            itag: format.itag as i64,
            expires_in_seconds: expires,
            loudness_db: format.loudness_db,
            client: key.to_owned(),
        });
    }
    Err(ResolveError::NoStream { video_id: video_id.to_owned() })
}

/// Resolve through the direct clients, then fall back to rustypipe for the whole video id.
pub async fn resolve_stream(
    it: &InnerTube,
    clients: &Clients,
    video_id: &str,
    quality: AudioQuality,
) -> Result<ResolvedStream, ResolveError> {
    let fetch = |key: &'static str, id: String| {
        let it = it.clone();
        async move {
            let client = clients.get(key)?;
            it.player(client, &id, None, None, None).await.ok()
        }
    };
    match resolve_with(clients, fetch, video_id, quality).await {
        Ok(stream) => Ok(stream),
        Err(ResolveError::NoStream { .. }) => {
            let prefer_high = matches!(quality, AudioQuality::High | AudioQuality::Auto);
            match rustypipe_fallback::resolve(video_id, prefer_high).await {
                Ok(candidate) => Ok(ResolvedStream {
                    video_id: video_id.to_owned(),
                    url: candidate.url,
                    headers: BTreeMap::new(),
                    itag: candidate.itag as i64,
                    expires_in_seconds: candidate.expires_in_seconds as i64,
                    loudness_db: candidate.loudness_db.map(f64::from),
                    client: "rustypipe".to_owned(),
                }),
                Err(_) => Err(ResolveError::NoStream {
                    video_id: video_id.to_owned(),
                }),
            }
        }
        Err(other) => Err(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Clients;

    fn player_json(url: Option<&str>, status: &str) -> serde_json::Value {
        serde_json::json!({
            "playabilityStatus": { "status": status },
            "streamingData": {
                "expiresInSeconds": "21600",
                "adaptiveFormats": [{
                    "itag": 251,
                    "url": url,
                    "mimeType": "audio/webm; codecs=opus",
                    "bitrate": 128000,
                    "audioQuality": "AUDIO_QUALITY_MEDIUM",
                    "audioChannels": 2,
                    "loudnessDb": -5.5
                }]
            }
        })
    }

    fn fetch_from(
        responses: Vec<serde_json::Value>,
    ) -> impl Fn(&'static str, String) -> std::future::Ready<Option<PlayerResponse>> {
        let queue = std::sync::Mutex::new(std::collections::VecDeque::from(responses));
        move |_key, _id| {
            let parsed = queue
                .lock()
                .unwrap()
                .pop_front()
                .and_then(|v| serde_json::from_value(v).ok());
            std::future::ready(parsed)
        }
    }

    #[tokio::test]
    async fn direct_url_is_returned_with_client_user_agent() {
        let clients = Clients::bundled();
        let got = resolve_with(
            &clients,
            fetch_from(vec![player_json(Some("https://a.test/audio"), "OK")]),
            "vid",
            AudioQuality::Auto,
        )
        .await
        .unwrap();
        assert_eq!(got.url, "https://a.test/audio");
        assert_eq!(got.itag, 251);
        assert_eq!(got.client, "VISIONOS");
        assert!(got.headers.contains_key("User-Agent"));
        assert_eq!(got.expires_in_seconds, 21_600);
        assert_eq!(got.loudness_db, Some(-5.5));
    }

    #[tokio::test]
    async fn ciphered_format_is_skipped_and_next_client_tried() {
        let clients = Clients::bundled();
        let got = resolve_with(
            &clients,
            fetch_from(vec![
                player_json(None, "OK"),
                player_json(Some("https://b.test/audio"), "OK"),
            ]),
            "vid",
            AudioQuality::Auto,
        )
        .await
        .unwrap();
        assert_eq!(got.url, "https://b.test/audio");
        assert_eq!(got.client, "ANDROID_VR_1_65_10");
    }

    #[tokio::test]
    async fn non_ok_playability_falls_through() {
        let clients = Clients::bundled();
        let got = resolve_with(
            &clients,
            fetch_from(vec![
                player_json(Some("https://dead.test"), "LOGIN_REQUIRED"),
                player_json(Some("https://live.test"), "OK"),
            ]),
            "vid",
            AudioQuality::Auto,
        )
        .await
        .unwrap();
        assert_eq!(got.url, "https://live.test");
    }

    #[tokio::test]
    async fn missing_streaming_data_falls_through() {
        let clients = Clients::bundled();
        let got = resolve_with(
            &clients,
            fetch_from(vec![
                serde_json::json!({ "playabilityStatus": { "status": "OK" } }),
                player_json(Some("https://live.test"), "OK"),
            ]),
            "vid",
            AudioQuality::Auto,
        )
        .await
        .unwrap();
        assert_eq!(got.url, "https://live.test");
    }

    #[tokio::test]
    async fn missing_expiry_falls_through() {
        let clients = Clients::bundled();
        let mut no_expiry = player_json(Some("https://dead.test"), "OK");
        no_expiry["streamingData"]
            .as_object_mut()
            .unwrap()
            .remove("expiresInSeconds");
        let got = resolve_with(
            &clients,
            fetch_from(vec![no_expiry, player_json(Some("https://live.test"), "OK")]),
            "vid",
            AudioQuality::Auto,
        )
        .await
        .unwrap();
        assert_eq!(got.url, "https://live.test");
    }

    #[tokio::test]
    async fn all_clients_failing_returns_no_stream() {
        let clients = Clients::bundled();
        let err = resolve_with(
            &clients,
            fetch_from(vec![
                player_json(None, "LOGIN_REQUIRED"),
                player_json(None, "LOGIN_REQUIRED"),
                player_json(None, "LOGIN_REQUIRED"),
            ]),
            "vid",
            AudioQuality::Auto,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ResolveError::NoStream { .. }));
    }
}
