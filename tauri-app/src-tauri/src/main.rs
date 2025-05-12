//! Tauri desktop shell for Oakley SRS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, GlobalShortcutManager};
use anyhow::Result;

// internal crates
use llm::gen_card;
use scheduler::{Scheduler, ReviewOutcome};
use data::{DbPool, insert_card};

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

#[tauri::command]
async fn generate_card(app: tauri::AppHandle, db: tauri::State<'_, DbPool>, text: String) -> Result<CardJson, String> {
    // Call OpenAI via llm crate
    let fields = gen_card(&text).await.map_err(|e| e.to_string())?;

    let db_card = data::CardJson {
        id: 0,
        front: fields.front.clone(),
        back: fields.back.clone(),
        tags: fields.tags.clone(),
    };

    let id = insert_card(&db, &db_card, None).map_err(|e| e.to_string())?;

    let card_json = CardJson {
        id,
        front: fields.front,
        back: fields.back,
        tags: fields.tags,
    };

    // Notify UI
    let _ = app.emit_all("card_created", &card_json);

    Ok(card_json)
}

// legacy background capture (no longer used)
#[allow(dead_code)]
fn spawn_background(_app: &tauri::AppHandle) {
    // ... existing code ...
    // Register global shortcut Cmd+Shift+P (less likely to conflict)
    // app.global_shortcut_manager().register("Cmd+Shift+,", move || {
}

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            // Initialise database and share via state
            let db = data::new_pool("oakley.db")?;
            app.manage(db.clone());

            // Kick off scheduler loop
            let (tx, _rx) = tokio::sync::mpsc::channel::<ReviewOutcome>(32);
            tauri::async_runtime::spawn(Scheduler::new(db.clone(), tx).run());

            // Register global shortcut Cmd+Shift+P (less likely to conflict)
            let handle = app.handle();
            app.global_shortcut_manager().register("Cmd+Shift+P", move || {
                let _ = handle.emit_all("hotkey", ());
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![generate_card, accept_card, discard_card])
        .run(tauri::generate_context!())
        .expect("error while running Oakley");
} 