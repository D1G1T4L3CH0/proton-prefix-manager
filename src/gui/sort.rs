use crate::core::models::GameInfo;
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameSortKey {
    /// Sort by game name
    Name,
    /// Sort by prefix modification time
    LastUpdated,
    /// Sort by last played timestamp
    LastPlayed,
    /// Sort by Steam AppID
    AppId,
    /// Sort by configured Proton version
    ProtonVersion,
}

impl Default for GameSortKey {
    fn default() -> Self {
        GameSortKey::LastPlayed
    }
}

impl GameSortKey {
    pub fn label(&self) -> &'static str {
        match self {
            GameSortKey::Name => "Name",
            GameSortKey::LastUpdated => "Last Updated",
            GameSortKey::LastPlayed => "Last Played",
            GameSortKey::AppId => "AppID",
            GameSortKey::ProtonVersion => "Proton Version",
        }
    }
}

pub fn compare_games(a: &GameInfo, b: &GameInfo, key: GameSortKey) -> Ordering {
    match key {
        GameSortKey::Name => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
        GameSortKey::LastUpdated => a.modified().cmp(&b.modified()),
        GameSortKey::LastPlayed => a.last_played().cmp(&b.last_played()),
        GameSortKey::AppId => a.app_id().cmp(&b.app_id()),
        GameSortKey::ProtonVersion => Ordering::Equal,
    }
}

pub fn sort_games(games: &mut [GameInfo], key: GameSortKey, descending: bool) {
    games.sort_by(|a, b| compare_games(a, b, key));
    if descending {
        games.reverse();
    }
}
