//! Binary entry-point that wires the pipeline together.

use anyhow::Result;
use capture::CaptureEvent;
use llm::{gen_card_from_image};
use scheduler::{Scheduler, ReviewOutcome};
use tokio::{select, sync::mpsc};
use tracing::{info, warn};
use notify_rust::Notification;

#[tokio::main]
async fn main() -> Result<()> {
    utils::log::init();

    // ── bootstrap shared state ──
    let db = data::new_pool("oakley.db")?;
    let (cap_tx, mut cap_rx) = mpsc::channel::<CaptureEvent>(16);
    let (rev_tx, mut rev_rx) = mpsc::channel::<ReviewOutcome>(32);
    let scheduler = Scheduler::new(db.clone(), rev_tx.clone());

    // ── task: global hot-key + screenshot capture ──
    tokio::spawn(capture::listen_and_capture(cap_tx.clone()));

    // ── task: scheduler tick every minute ──
    tokio::spawn(scheduler.run());

    // ── main bus ──
    loop {
        select! {
            Some(evt) = cap_rx.recv() => {
                info!("📸 Capture event received: region={:?}", evt.region);
                // Encode image to PNG bytes for OpenAI
                let mut png_bytes = Vec::new();
                image::codecs::png::PngEncoder::new(&mut png_bytes)
                    .encode(
                        &evt.image,
                        evt.image.width(),
                        evt.image.height(),
                        image::ColorType::Rgba8,
                    )?;

                let card = gen_card_from_image(&png_bytes).await?;
                info!("🧠 Generated card from image");
                let card_json = data::CardJson { id: 0, front: card.front, back: card.back, tags: card.tags };
                let new_id = data::insert_card(&db, &card_json, evt.path.as_deref())?;
                info!("inserted card id={new_id}");

                // Fire a system notification so the user knows card was created
                let _ = Notification::new()
                    .summary("Oakley – Card Saved")
                    .body(&format!("Card #{new_id} created from screenshot."))
                    .icon("dialog-information")
                    .show();
            }
            Some(outcome) = rev_rx.recv() => {
                // TODO: persist outcome
                warn!(?outcome, "received review outcome – persistence TBD");
            }
        }
    }
} 