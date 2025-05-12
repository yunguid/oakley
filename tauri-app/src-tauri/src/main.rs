//! Tauri desktop shell for Oakley SRS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, GlobalShortcutManager};
use anyhow::Result;

// internal crates
use llm::{gen_card, gen_card_from_image};
use scheduler::{Scheduler, ReviewOutcome};
use data::{DbPool, insert_card};
use capture::CaptureEvent;
use tracing::{info, error, warn};
use get_selected_text::get_selected_text;

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

#[tauri::command]
async fn create_card_from_selection(app: tauri::AppHandle, db: tauri::State<'_, DbPool>) -> Result<(), String> {
    info!("Attempting to get selected text...");
    match get_selected_text() {
        Ok(selected_text) => {
            if selected_text.trim().is_empty() {
                warn!("No text selected or retrieved.");
                // Optionally, emit an event to the frontend to show a notification
                // app.emit_all("info_message", "No text selected").ok();
                return Ok(());
            }
            info!(length = selected_text.len(), "Got selected text, generating card.");
            let _ = app.emit_all("hotkey", ()); // Reuse existing event to show spinner

            // Use existing llm::gen_card function
            match gen_card(&selected_text).await {
                Ok(fields) => {
                    let db_card = data::CardJson {
                        id: 0,
                        front: fields.front.clone(),
                        back: fields.back.clone(),
                        tags: fields.tags.clone(),
                    };
                    match insert_card(&db, &db_card, None) {
                        Ok(id) => {
                            info!(id, "🧠 Card saved from selection");
                            let card_json = CardJson {
                                id,
                                front: fields.front,
                                back: fields.back,
                                tags: fields.tags,
                            };
                            let _ = app.emit_all("card_created", &card_json);
                        }
                        Err(e) => {
                            error!(?e, "Failed to insert card from selection into DB");
                            // Optionally notify frontend of DB error
                            // app.emit_all("error_message", format!("DB Error: {}", e)).ok();
                            return Err(format!("DB Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    error!(?e, "Failed to generate card from selected text via LLM");
                    // Optionally notify frontend of LLM error
                    // app.emit_all("error_message", format!("LLM Error: {}", e)).ok();
                    return Err(format!("LLM Error: {}", e));
                }
            }
        }
        Err(_) => {
            error!("Failed to get selected text");
            // Optionally notify frontend
            // app.emit_all("error_message", "Could not get selected text").ok();
            // This might indicate a permissions issue on macOS.
            #[cfg(all(target_os = "macos", feature = "macos-permissions"))]
            {
                warn!("Ensure Accessibility Permissions are granted for Oakley.");
                // Consider prompting for permissions if not granted
                // if !macos_accessibility_client::accessibility::application_is_trusted_with_prompt() {
                //     warn!("Accessibility permissions denied or prompt failed.");
                // }
            }
            return Err("Failed to get selected text".to_string());
        }
    }
    Ok(())
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
            let db_clone_capture = db.clone();
            let db_clone_selection = db.clone();

            // --- Global shortcut for Image Capture (Cmd+Shift+Comma) ---
            app.global_shortcut_manager().register("Cmd+Shift+Comma", move || {
                let db = db_clone_capture.clone();
                let async_handle = shortcut_handle.clone();
                // ... spawn capture task ...
            })?;

            // --- Global shortcut for Text Selection (Cmd+Shift+Period) ---
            // Using Cmd+Shift+. as Cmd+Shift+< might be awkward / require alias
            let shortcut_handle_sel = app.handle();
            app.global_shortcut_manager().register("Cmd+Shift+.", move || {
                info!("Text selection shortcut triggered");
                let db_sel = db_clone_selection.clone();
                let async_handle_sel = shortcut_handle_sel.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = create_card_from_selection(async_handle_sel.clone(), tauri::State::from(&db_sel)).await {
                        error!(error = %e, "Error processing text selection shortcut");
                        // Optionally notify the frontend about the overall failure
                        // async_handle_sel.emit_all("error_message", format!("Failed: {}", e)).ok();
                    }
                });
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_card,
            accept_card,
            discard_card,
            create_card_from_selection // <-- Register new command
        ])
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