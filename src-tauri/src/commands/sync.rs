use crate::entities::operations;
use crate::entities::tag_rules;
use rusqlite::Connection;
use std::ops::DerefMut;
use std::sync::Mutex;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
struct SyncResult {
    rules_applied: usize,
    duplicates_refreshed: usize,
}

impl SyncResult {
    pub fn new(rules_applied: usize, duplicates_refreshed: usize) -> Self {
        SyncResult { rules_applied, duplicates_refreshed }
    }
}

#[tauri::command]
pub(crate) fn sync(conn_state: State<'_, Mutex<Connection>>) -> String {
    let mut conn = conn_state
        .inner()
        .lock()
        .expect("Could not retrieve database connection");
    let mut conn = conn.deref_mut();

    let rules_applied = tag_rules::apply_rules(&conn);
    let duplicates = operations::refresh_statuses_with_hashes(&mut conn);

    serde_json::to_string(&SyncResult::new(rules_applied, duplicates)).unwrap()
}
