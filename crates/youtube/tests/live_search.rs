#![cfg(feature = "live-tests")]

use tempo_youtube::{search_songs, InnerTube, Session};

#[tokio::test]
async fn live_search_returns_songs_for_ten_queries() {
    let it = InnerTube::new(Session::default(), None).unwrap();
    for query in [
        "Arijit Singh",
        "Taylor Swift",
        "Prateek Kuhad",
        "lofi hip hop",
        "Beethoven Moonlight Sonata",
        "Daft Punk",
        "Kendrick Lamar",
        "Anoushka Shankar",
        "Radiohead",
        "SPB hits",
    ] {
        let songs = search_songs(&it, query).await.unwrap();
        assert!(!songs.is_empty(), "no songs for {query}");
        assert!(songs.iter().all(|s| !s.is_video));
    }
}
