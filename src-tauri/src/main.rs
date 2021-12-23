#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use rusqlite::Connection;
use std::ops::Deref;
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
            import_ofx,
            import_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod embedded {
    refinery::embed_migrations!("src/migrations/");
}

#[tauri::command]
fn get_operations(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    serde_json::to_string(&operations::find_all(&conn)).expect("Could not serialize Operations properly")
}

#[tauri::command]
fn get_bank_accounts(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    serde_json::to_string(&bank_accounts::find_all(&conn)).expect("Could not serialize BankAccount properly")
}

#[tauri::command]
fn save_bank_account(conn_state: State<'_, Mutex<Connection>>, bank_account: String) {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    let bank_account_entity: BankAccount = serde_json::from_str(&bank_account).unwrap();
    bank_accounts::save(conn, bank_account_entity);
}

#[tauri::command]
fn get_tags(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    serde_json::to_string(&tags::find_all(&conn)).expect("Could not serialize Tag properly")
}

#[tauri::command]
fn get_tag_rules(conn_state: State<'_, Mutex<Connection>>) -> String {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    serde_json::to_string(&tag_rules::find_all(&conn)).expect("Could not serialize Tag rules properly")
}

#[tauri::command]
fn save_tag(conn_state: State<'_, Mutex<Connection>>, tag: String) {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    let tag_entity: Tag = serde_json::from_str(&tag).unwrap();
    tags::save(conn, tag_entity);
}

#[tauri::command]
fn save_tag_rule(conn_state: State<'_, Mutex<Connection>>, tag_rule: String) {
    let conn = conn_state.inner().lock().expect("Could not retrieve connection");
    let conn = conn.deref();

    let tag_rule_entity: TagRule = serde_json::from_str(&tag_rule).unwrap();
    tag_rules::save(conn, tag_rule_entity);
}

#[tauri::command]
fn import_ofx(_conn_state: State<'_, Mutex<Connection>>, file_content: String) {
    let splitted = file_content.split_once("<OFX>").unwrap();

    let headers: String = splitted.0.to_string();
    let mut xml_body: String = String::from("<OFX>");
    xml_body.push_str(splitted.1);
    let xml_body = xml_body;

    let sgml = sgmlish::Parser::builder()
        .lowercase_names()
        .parse(&xml_body)
        .expect("Could not parse SGML content from OFX data");
    let sgml = sgmlish::transforms::normalize_end_tags(sgml)
        .expect("Could not normalize SGML content from OFX data");

    println!("Headers:\n{}\n\nXML body:\n{}", headers, xml_body);

    dbg!(sgml);
}

#[tauri::command]
fn import_csv(
    _conn_state: State<'_, Mutex<Connection>>,
    file_content: String,
    number_of_lines_to_remove: u16,
    csv_separator: String,
    csv_delimiter: String,
    csv_escape_character: String,
) {
    println!("CSV File:\n{}", file_content);
    println!("number_of_lines_to_remove:\n{}", number_of_lines_to_remove);
    println!("csv_separator:\n{}", csv_separator);
    println!("csv_delimiter:\n{}", csv_delimiter);
    println!("csv_escape_character:\n{}", csv_escape_character);
}