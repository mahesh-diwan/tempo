use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStream {
    pub video_id: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub itag: i64,
    pub expires_in_seconds: i64,
    pub loudness_db: Option<f64>,
    pub client: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_are_ordered_and_round_trip() {
        let mut headers = BTreeMap::new();
        headers.insert("User-Agent".into(), "test-agent".into());
        let stream = ResolvedStream {
            video_id: "abc".into(),
            url: "https://example.test/audio".into(),
            headers,
            itag: 251,
            expires_in_seconds: 21_600,
            loudness_db: Some(-6.5),
            client: "VISIONOS".into(),
        };
        assert_eq!(stream.headers.get("User-Agent").unwrap(), "test-agent");
        assert_eq!(stream.expires_in_seconds, 21_600);
    }
}
