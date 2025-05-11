//! Tauri desktop shell for Oakley SRS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct CardJson {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

#[tauri::command]
fn accept_card(card: CardJson) -> Result<(), String> {
    // TODO: send acknowledgement back to core process via IPC
    println!("Card accepted: {:?}", card);
    Ok(())
}

#[tauri::command]
fn discard_card(card_id: i64) -> Result<(), String> {
    println!("Card discarded: {}", card_id);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![accept_card, discard_card])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 