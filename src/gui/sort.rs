use crate::core::models::GameInfo;
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameSortKey {
    Name,
    Modified,
    LastPlayed,
    AppId,
}

pub fn compare_games(a: &GameInfo, b: &GameInfo, key: GameSortKey) -> Ordering {
    match key {
        GameSortKey::Name => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
        GameSortKey::Modified => a.modified().cmp(&b.modified()),
        GameSortKey::LastPlayed => a.last_played().cmp(&b.last_played()),
        GameSortKey::AppId => a.app_id().cmp(&b.app_id()),
    }
}

pub fn sort_games(games: &mut [GameInfo], key: GameSortKey, descending: bool) {
    games.sort_by(|a, b| compare_games(a, b, key));
    if descending {
        games.reverse();
    }
}
