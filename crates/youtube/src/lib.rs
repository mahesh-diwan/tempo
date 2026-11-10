//! Anonymous YouTube Music InnerTube transport, client identities, models, endpoints, and the
//! rustypipe fallback.
//!
//! v1 boundary: no account, cookie, cipher, PoToken, or `WEB_REMIX` stream path. The crate is
//! unit-testable against JSON fixtures with no network. `WEB_REMIX` is used only as the search
//! metadata client identity, never as a stream source.

pub mod clients;
pub mod endpoints;
pub mod models;
pub mod rustypipe_fallback;
pub mod search;
pub mod transport;

pub use clients::{
    Clients, YouTubeClient, LYRICS_TIMED_CLIENT, MAIN_CLIENT, METADATA_CLIENT,
    STREAM_FALLBACK_ORDER, UPLOAD_FALLBACK_ORDER,
};
pub use models::browse::{
    AlbumPage, ArtistCarousel, ArtistPage, BrowseItem, HomePage, PlaylistContinuation,
    PlaylistPage, PlaylistSort, SearchCardPage, SearchResults, Section, SortMenu,
};
pub use models::context::Locale;
pub use models::lyrics::{PlainLyrics, TimedLyricLine};
pub use models::metadata::{
    AccountIdentity, AccountInfo, NextResult, Rating, SearchResult, SongItem,
};
pub use models::player::{
    find_format, find_video_format, AudioQuality, Format, PlaybackTracking, PlayerResponse,
    StreamingData,
};
pub use rustypipe_fallback::{FallbackError, StreamCandidate};
pub use search::{parse_song_items, search_songs, YoutubeError};
pub use transport::{cookie_sapisid, generate_cpn, Error, InnerTube, Session};
