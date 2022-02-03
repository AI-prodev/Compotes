#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use rusqlite::Connection;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Mutex;
use tauri::State;
use crate::entities::operations;
use crate::entities::bank_accounts;
use crate::entities::tags;
use crate::entities::tag_rules;
use crate::entities::tags::Tag;
use crate::entities::tag_rules::TagRule;
use crate::db::get_database_connection;
use crate::entities::bank_accounts::BankAccount;
use crate::entities::operations::Operation;

mod db;
mod config;

mod entities {
    pub(crate) mod operations;
    pub(crate) mod bank_accounts;
    pub(crate) mod tags;
    pub(crate) mod tag_rules;
}

mod structs {
    pub(crate) mod operation_state;
}

fn main() {
    let mut conn = get_database_connection();

    embedded::migrations::runner().run(&mut conn).expect("Could not execute database migrations.");

    tauri::Builder::default()
        .manage(Mutex::new(conn))
        .invoke_handler(tauri::generate_handler![
            get_operations,
            get_bank_accounts,
            save_bank_account,
            get_tags,
            get_tag_rules,
            save_tag,
            save_tag_rule,
            import_operations,
            sync,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}

mod embedded {
    refinery::embed_migrations!("src/migrations/");
}

#[tauri::command]
fn get_operations(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    serde_json::to_string(&operations::find_all(&conn)).expect("Could not serialize Operations properly")
}

#[tauri::command]
fn get_bank_accounts(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    serde_json::to_string(&bank_accounts::find_all(&conn)).expect("Could not serialize BankAccount properly")
}

#[tauri::command]
fn save_bank_account(conn_state: State<'_, Mutex<Connection>>, bank_account: String) {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    let bank_account_entity: BankAccount = serde_json::from_str(&bank_account).unwrap();
    bank_accounts::save(conn, bank_account_entity);
}

#[tauri::command]
fn get_tags(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    serde_json::to_string(&tags::find_all(&conn)).expect("Could not serialize Tag properly")
}

#[tauri::command]
fn get_tag_rules(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    serde_json::to_string(&tag_rules::find_all(&conn)).expect("Could not serialize Tag rules properly")
}

#[tauri::command]
fn save_tag(conn_state: State<'_, Mutex<Connection>>, tag: String) {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    let tag_entity: Tag = serde_json::from_str(&tag).unwrap();
    tags::save(conn, tag_entity);
}

#[tauri::command]
fn save_tag_rule(conn_state: State<'_, Mutex<Connection>>, tag_rule: String) {
    let conn = conn_state.inner().lock().expect("Could not retrieve database connection");
    let conn = conn.deref();

    let tag_rule_entity: TagRule = serde_json::from_str(&tag_rule).unwrap();
    tag_rules::save(conn, tag_rule_entity);
}

#[tauri::command]
fn import_operations(
    _conn_state: State<'_, Mutex<Connection>>,
    operations: Vec<Operation>
) {
    let mut conn = _conn_state.inner().lock().expect("Could not retrieve database connection");
    let mut conn = conn.deref_mut();

    operations::insert_all(&mut conn, operations);
}

#[derive(Clone, serde::Serialize)]
struct Message {
    message: String,
    message_type: String,
}

#[tauri::command]
fn sync() -> String {
    operations::refresh_statuses_with_hashes();
    tag_rules::apply_rules();
    std::thread::sleep(std::time::Duration::from_secs(1));

    return "1".to_string();
}
