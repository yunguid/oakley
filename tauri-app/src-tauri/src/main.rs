//! Tauri desktop shell for Oakley SRS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, GlobalShortcutManager};
use anyhow::Result;

// internal crates
use llm::{gen_card, gen_card_from_image};
use scheduler::{Scheduler, ReviewOutcome};
use data::{DbPool, insert_card};
use capture::CaptureEvent;
use tracing::{info, error};

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
            let (rev_tx, _rev_rx) = tokio::sync::mpsc::channel::<ReviewOutcome>(32);
            tauri::async_runtime::spawn(Scheduler::new(db.clone(), rev_tx.clone()).run());

            // ── Global shortcut Cmd+Shift+<  ──
            let shortcut_handle = app.handle();
            let db_clone = db.clone();

            app.global_shortcut_manager().register("Cmd+Shift+Comma", move || {
                let db = db_clone.clone();

                // Clone separately for the async task so we don't move the same handle twice.
                let async_handle = shortcut_handle.clone();
                tauri::async_runtime::spawn(async move {
                    match capture::capture_screen() {
                        Ok(evt) => {
                            // User finished selecting screen area – now notify UI spinner.
                            let _ = async_handle.emit_all("hotkey", ());

                            if let Err(e) = process_capture(evt, &db, &async_handle).await {
                                error!(?e, "failed to process capture");
                            }
                        }
                        Err(e) => error!(?e, "capture error"),
                    }
                });
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![generate_card, accept_card, discard_card])
        .run(tauri::generate_context!())
        .expect("error while running Oakley");
}

async fn process_capture(evt: CaptureEvent, db: &DbPool, app_handle: &tauri::AppHandle) -> Result<()> {
    // Always build PNG from in-memory image to avoid temp-file lifetime issues.
    let mut png_bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_bytes)
        .encode(
            &evt.image,
            evt.image.width(),
            evt.image.height(),
            image::ColorType::Rgba8,
        )?;

    info!(size = png_bytes.len(), "📸 Screenshot bytes prepared");

    let fields = gen_card_from_image(&png_bytes).await?;

    let db_card = data::CardJson {
        id: 0,
        front: fields.front.clone(),
        back: fields.back.clone(),
        tags: fields.tags.clone(),
    };

    let id = insert_card(db, &db_card, evt.path.as_deref())?;
    info!(id, "🧠 Card saved");

    let card_json = CardJson {
        id,
        front: fields.front,
        back: fields.back,
        tags: fields.tags,
    };

    let _ = app_handle.emit_all("card_created", &card_json);

    Ok(())
} 