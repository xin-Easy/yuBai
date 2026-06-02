use crate::{
    domain::runtime::{BookmarkSyncResult, BrowserBookmark},
    infra::db::{
        database::Database,
        support::{map_bookmark_row, BrowserBookmarkRow, NewBrowserBookmarkRow},
    },
    error::AppError,
};
use diesel::{dsl::count_star, prelude::*};

impl Database {
    pub fn list_bookmarks(&self) -> Result<Vec<BrowserBookmark>, AppError> {
        use crate::infra::db::schema::browser_bookmarks::dsl as bookmarks;

        let mut conn = self.lock()?;
        let rows = bookmarks::browser_bookmarks
            .select(BrowserBookmarkRow::as_select())
            .order((bookmarks::sort_order.asc(), bookmarks::id.asc()))
            .load::<BrowserBookmarkRow>(&mut *conn)?;

        Ok(rows.into_iter().map(map_bookmark_row).collect())
    }

    pub fn save_bookmarks(&self, items: Vec<BrowserBookmark>) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_bookmarks::dsl as bookmarks;

        let mut conn = self.lock()?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(bookmarks::browser_bookmarks).execute(conn)?;
            for (index, item) in items.iter().enumerate() {
                diesel::insert_into(bookmarks::browser_bookmarks)
                    .values(&NewBrowserBookmarkRow {
                        name: item.name.trim(),
                        url: item.url.trim(),
                        open_on_start: if item.open_on_start { 1 } else { 0 },
                        sort_order: index as i32,
                    })
                    .execute(conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn reset_bookmarks(&self) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_bookmarks::dsl as bookmarks;

        let mut conn = self.lock()?;
        diesel::delete(bookmarks::browser_bookmarks).execute(&mut *conn)?;
        Ok(())
    }

    pub fn bookmark_sync(&self) -> Result<BookmarkSyncResult, AppError> {
        let mut conn = self.lock()?;
        let total = crate::infra::db::schema::browser_profiles::table
            .select(count_star())
            .first::<i64>(&mut *conn)?;

        Ok(BookmarkSyncResult {
            total: total as i32,
            synced: 0,
            skipped: 0,
            failed: 0,
            skipped_list: Vec::new(),
            failed_list: Vec::new(),
        })
    }
}
