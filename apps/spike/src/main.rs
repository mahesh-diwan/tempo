//! Milestone 2 extraction gate harness.
//!
//! Runs the 10-query, 50-track sample: search, resolve, play 10 seconds. Writes a JSON report
//! and exits 0 when the gate passes, 2 when it fails.

mod report;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tempo_model::ResolvedStream;
use tempo_player::{Player, PlayerEvent};
use tempo_youtube::{
    resolve_stream, search_songs, AudioQuality, Clients, InnerTube, Session,
};

use crate::report::{SpikeReport, TrackReport};

const QUERIES: [&str; 10] = [
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
];

struct Args {
    tracks_per_query: usize,
    play_seconds: u64,
    output: PathBuf,
    no_play: bool,
}

impl Args {
    fn parse() -> Self {
        let mut tracks_per_query = 5usize;
        let mut play_seconds = 10u64;
        let mut output = PathBuf::from("spike-report.json");
        let mut no_play = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--tracks-per-query" => {
                    tracks_per_query = args
                        .next()
                        .and_then(|v| v.parse().ok())
                        .expect("--tracks-per-query needs a number");
                }
                "--play-seconds" => {
                    play_seconds = args
                        .next()
                        .and_then(|v| v.parse().ok())
                        .expect("--play-seconds needs a number");
                }
                "--output" => {
                    output = args
                        .next()
                        .map(PathBuf::from)
                        .expect("--output needs a path");
                }
                "--no-play" => no_play = true,
                other => panic!("unknown argument: {other}"),
            }
        }
        Args { tracks_per_query, play_seconds, output, no_play }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let it = InnerTube::new(Session::default(), None).expect("InnerTube");
    let clients = Clients::bundled();

    let cache_dir = std::env::temp_dir().join("tempo-spike-cache");
    std::fs::create_dir_all(&cache_dir).expect("cache dir");
    let mut player = if args.no_play {
        None
    } else {
        Some(Player::new(cache_dir.to_str().unwrap()).expect("libmpv"))
    };
    let mut events = player.as_mut().and_then(|p| p.take_events());

    let mut report = SpikeReport {
        queries: QUERIES.iter().map(|q| q.to_string()).collect(),
        tracks: Vec::new(),
    };

    for query in QUERIES {
        let songs = match search_songs(&it, query).await {
            Ok(songs) => songs,
            Err(e) => {
                report.tracks.push(TrackReport {
                    query: query.to_string(),
                    video_id: String::new(),
                    title: String::new(),
                    status: "search_failed".into(),
                    client: None,
                    itag: None,
                    elapsed_ms: 0,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };
        let mut taken = 0;
        for song in songs.iter().filter(|s| !s.is_video) {
            if taken >= args.tracks_per_query {
                break;
            }
            taken += 1;
            let started = Instant::now();
            match resolve_stream(&it, &clients, &song.video_id, AudioQuality::Auto).await {
                Err(e) => report.tracks.push(TrackReport {
                    query: query.to_string(),
                    video_id: song.video_id.clone(),
                    title: song.title.clone(),
                    status: "failed".into(),
                    client: None,
                    itag: None,
                    elapsed_ms: started.elapsed().as_millis(),
                    error: Some(e.to_string()),
                }),
                Ok(stream) => {
                    let client = stream.client.clone();
                    let itag = stream.itag;
                    if args.no_play {
                        report.tracks.push(TrackReport {
                            query: query.to_string(),
                            video_id: song.video_id.clone(),
                            title: song.title.clone(),
                            status: "resolved".into(),
                            client: Some(client),
                            itag: Some(itag),
                            elapsed_ms: started.elapsed().as_millis(),
                            error: None,
                        });
                    } else {
                        match play_track(
                            player.as_ref().unwrap(),
                            events.as_mut().unwrap(),
                            &stream,
                            args.play_seconds,
                        )
                        .await
                        {
                            Ok(_) => report.tracks.push(TrackReport {
                                query: query.to_string(),
                                video_id: song.video_id.clone(),
                                title: song.title.clone(),
                                status: "played".into(),
                                client: Some(client),
                                itag: Some(itag),
                                elapsed_ms: started.elapsed().as_millis(),
                                error: None,
                            }),
                            Err(e) => report.tracks.push(TrackReport {
                                query: query.to_string(),
                                video_id: song.video_id.clone(),
                                title: song.title.clone(),
                                status: "failed".into(),
                                client: Some(client),
                                itag: Some(itag),
                                elapsed_ms: started.elapsed().as_millis(),
                                error: Some(e),
                            }),
                        }
                    }
                }
            }
        }
        if taken == 0 {
            report.tracks.push(TrackReport {
                query: query.to_string(),
                video_id: String::new(),
                title: String::new(),
                status: "search_empty".into(),
                client: None,
                itag: None,
                elapsed_ms: 0,
                error: None,
            });
        }
    }

    let json = serde_json::to_string_pretty(&report).expect("report json");
    std::fs::write(&args.output, json).expect("write report");

    let pass = report.playable();
    let fail = report.tracks.len() - pass;
    let queries = report.queries_with_results();
    let gate = report.gate_passes();
    println!(
        "pass={pass} fail={fail} queries={queries}/{} gate={}",
        report.queries.len(),
        if gate { "pass" } else { "fail" }
    );
    std::process::exit(if gate { 0 } else { 2 });
}

async fn play_track(
    player: &Player,
    events: &mut tokio::sync::mpsc::UnboundedReceiver<PlayerEvent>,
    stream: &ResolvedStream,
    play_seconds: u64,
) -> Result<f64, String> {
    let mut headers = HashMap::new();
    for (k, v) in &stream.headers {
        headers.insert(k.clone(), v.clone());
    }
    player
        .load(&stream.url, &headers, stream.loudness_db)
        .map_err(|e| e.to_string())?;

    let started = Instant::now();
    let mut playing = false;
    while started.elapsed() < Duration::from_secs(20) {
        match tokio::time::timeout(Duration::from_secs(5), events.recv()).await {
            Ok(Some(PlayerEvent::Playing(true))) => {
                playing = true;
                break;
            }
            Ok(Some(PlayerEvent::TrackFailed(e))) => return Err(e),
            Ok(Some(_)) => {}
            Ok(None) => return Err("player event channel closed".into()),
            Err(_) => {}
        }
    }
    if !playing {
        let _ = player.stop();
        return Err("no playback start within 20s".into());
    }

    tokio::time::sleep(Duration::from_secs(play_seconds)).await;
    let position = player.position().unwrap_or(0.0);
    player.stop().map_err(|e| e.to_string())?;
    if position >= play_seconds as f64 - 1.0 {
        Ok(position)
    } else {
        Err(format!("stalled at {position:.1}s"))
    }
}
