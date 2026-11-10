use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongItem {
    pub video_id: String,
    pub title: String,
    pub artists: String,
    pub album: Option<String>,
    pub duration: Option<String>,
    pub thumbnail: Option<String>,
    pub is_video: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_ascii_round_trip() {
        let song = SongItem {
            video_id: "abc123".into(),
            title: "तुम ही हो".into(),
            artists: "अरिजीत सिंह".into(),
            album: Some("Aashiqui 2".into()),
            duration: Some("4:22".into()),
            thumbnail: None,
            is_video: false,
        };
        let json = serde_json::to_string(&song).unwrap();
        let back: SongItem = serde_json::from_str(&json).unwrap();
        assert_eq!(song, back);
    }

    #[test]
    fn optional_fields_default_to_null() {
        let song: SongItem = serde_json::from_str(
            r#"{"video_id":"x","title":"t","artists":"a","album":null,"duration":null,"thumbnail":null,"is_video":false}"#,
        )
        .unwrap();
        assert!(song.album.is_none());
        assert!(!song.is_video);
    }
}
