//! Tauri desktop shell for Oakley SRS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, AppHandle};
use tokio::sync::mpsc;
use anyhow::Result;

// internal crates
use capture::CaptureEvent;
use ocr::extract_text;
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

fn spawn_background(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        // bootstrap shared state
        let db: DbPool = match data::new_pool("oakley.db") {
            Ok(db) => db,
            Err(e) => {
                eprintln!("DB init error: {e}");
                return;
            }
        };

        let (cap_tx, mut cap_rx) = mpsc::channel::<CaptureEvent>(16);
        let (rev_tx, mut _rev_rx) = mpsc::channel::<ReviewOutcome>(32);
        let scheduler = Scheduler::new(db.clone(), rev_tx.clone());

        // scheduler loop
        tauri::async_runtime::spawn(scheduler.run());

        // capture listener
        tauri::async_runtime::spawn(async move {
            if let Err(e) = capture::listen_and_capture(cap_tx).await {
                eprintln!("Capture listener error: {e}");
            }
        });

        // main bus
        while let Some(evt) = cap_rx.recv().await {
            // notify UI card generation started
            let _ = app_handle.emit_all("card_generating", ());

            // OCR
            let txt = match extract_text(&evt.image) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("OCR error: {e}");
                    continue;
                }
            };
            // LLM
            let card_fields = match gen_card(&txt).await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("LLM error: {e}");
                    continue;
                }
            };
            // convert to CardJson with dummy id for now
            let mut card_json = CardJson {
                id: 0,
                front: card_fields.front.clone(),
                back: card_fields.back.clone(),
                tags: card_fields.tags.clone(),
            };

            let db_card = data::CardJson {
                id: 0,
                front: card_fields.front,
                back: card_fields.back,
                tags: card_json.tags.clone(),
            };

            // DB insert
            match insert_card(&db, &db_card, evt.path.as_deref()) {
                Ok(id) => {
                    card_json.id = id;
                    let _ = app_handle.emit_all("card_created", &card_json);
                }
                Err(e) => {
                    eprintln!("DB insert error: {e}");
                    let _ = app_handle.emit_all("card_created", &card_json);
                }
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            spawn_background(&app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![accept_card, discard_card])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 