#![cfg(feature = "live-tests")]

use tempo_youtube::{
    resolve_stream, search_songs, AudioQuality, Clients, InnerTube, Session,
};

#[tokio::test]
async fn live_resolve_returns_a_direct_audio_url() {
    let it = InnerTube::new(Session::default(), None).unwrap();
    let clients = Clients::bundled();
    let songs = search_songs(&it, "Arijit Singh").await.unwrap();
    let song = songs
        .iter()
        .find(|s| !s.is_video)
        .expect("no song rows");
    let stream = resolve_stream(&it, &clients, &song.video_id, AudioQuality::Auto)
        .await
        .unwrap();
    assert!(stream.url.starts_with("http"), "url: {}", stream.url);
    assert!(stream.itag > 0);
}
