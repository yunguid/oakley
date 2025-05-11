//! Binary entry-point that wires the pipeline together.

use anyhow::Result;
use capture::CaptureEvent;
use ocr::extract_text;
use llm::gen_card;
use scheduler::{Scheduler, ReviewOutcome};
use tokio::{select, sync::mpsc};
use tracing::{info, warn};

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
                info!("got capture event (stub)");
                let txt = extract_text(&evt.image)?;
                let card = gen_card(&txt).await?;
                let card_json = data::CardJson { id: 0, front: card.front, back: card.back, tags: card.tags };
                let new_id = data::insert_card(&db, &card_json, evt.path.as_deref())?;
                info!("inserted card id={new_id}");
                // TODO: IPC → UI popup here
            }
            Some(outcome) = rev_rx.recv() => {
                // TODO: persist outcome
                warn!(?outcome, "received review outcome – persistence TBD");
            }
        }
    }
} 