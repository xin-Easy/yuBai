use crate::{
    app::state::AppState,
    domain::runtime::{BookmarkSyncResult, BrowserBookmark},
    error::AppError,
};

pub struct BookmarkService;

impl BookmarkService {
    pub fn list(&self, state: &AppState) -> Result<Vec<BrowserBookmark>, AppError> {
        state.repositories.database.list_bookmarks()
    }

    pub fn save(&self, state: &AppState, items: Vec<BrowserBookmark>) -> Result<(), AppError> {
        state.repositories.database.save_bookmarks(items)
    }

    pub fn reset(&self, state: &AppState) -> Result<(), AppError> {
        state.repositories.database.reset_bookmarks()
    }

    pub fn sync_to_profiles(&self, state: &AppState) -> Result<BookmarkSyncResult, AppError> {
        state.repositories.database.bookmark_sync()
    }
}
