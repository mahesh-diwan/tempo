//! Gate report types and pass rule.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackReport {
    pub query: String,
    pub video_id: String,
    pub title: String,
    pub status: String,
    pub client: Option<String>,
    pub itag: Option<i64>,
    pub elapsed_ms: u128,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpikeReport {
    pub queries: Vec<String>,
    pub tracks: Vec<TrackReport>,
}

impl SpikeReport {
    pub fn playable(&self) -> usize {
        self.tracks.iter().filter(|t| t.status == "played").count()
    }

    pub fn queries_with_results(&self) -> usize {
        let mut seen: Vec<&str> = Vec::new();
        for t in &self.tracks {
            if !matches!(t.status.as_str(), "search_empty" | "search_failed")
                && !seen.contains(&t.query.as_str())
            {
                seen.push(&t.query);
            }
        }
        seen.len()
    }

    pub fn gate_passes(&self) -> bool {
        self.queries_with_results() == self.queries.len()
            && self.tracks.len() == 50
            && self.playable() * 100 >= 90 * self.tracks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(query: &str, status: &str) -> TrackReport {
        TrackReport {
            query: query.to_owned(),
            video_id: "vid".to_owned(),
            title: "Title".to_owned(),
            status: status.to_owned(),
            client: None,
            itag: None,
            elapsed_ms: 1,
            error: None,
        }
    }

    fn report_with_played(played: usize) -> SpikeReport {
        let queries: Vec<String> = (0..10).map(|i| format!("q{i}")).collect();
        let mut tracks = Vec::new();
        for query in &queries {
            for _ in 0..5 {
                let status = if tracks.len() < played { "played" } else { "failed" };
                tracks.push(track(query, status));
            }
        }
        SpikeReport { queries, tracks }
    }

    #[test]
    fn gate_passes_at_45_of_50() {
        assert!(report_with_played(45).gate_passes());
    }

    #[test]
    fn gate_fails_at_44_of_50() {
        assert!(!report_with_played(44).gate_passes());
    }

    #[test]
    fn gate_fails_when_a_query_has_no_results() {
        let mut report = report_with_played(45);
        for t in report.tracks.iter_mut().take(5) {
            t.status = "search_empty".to_owned();
        }
        assert!(!report.gate_passes());
    }

    #[test]
    fn gate_fails_when_a_query_search_failed() {
        let mut report = report_with_played(45);
        for t in report.tracks.iter_mut().take(5) {
            t.status = "search_failed".to_owned();
        }
        assert!(!report.gate_passes());
    }

    #[test]
    fn report_round_trips_through_json() {
        let report = report_with_played(45);
        let json = serde_json::to_string(&report).unwrap();
        let back: SpikeReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.playable(), 45);
        assert!(back.gate_passes());
    }
}
