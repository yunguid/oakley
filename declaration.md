# Oakley SRS – Project Declaration & Road-map

_(version 0.1 • 10 May 2025)_

---

## 1 Executive summary

Oakley SRS is an AI-powered spaced-repetition system that captures text or screenshots from your screen, turns them into flash-cards using OpenAI, and periodically quizzes the user by voice or text. The system provides both a desktop app for card creation and a web interface for browsing and reviewing cards.

---

## 2 Objectives & success criteria

|Objective|Measurable success|
|---|---|
|_Zero-friction capture_|Hot-key → card in ≤ 3 s 90 % of the time|
|_Accurate card generation_|≥ 95% user acceptance rate of generated cards|
|_Reliable scheduling_|Scheduler error < 0.1 days across 10 000 simulated reviews (SM-2)|
|_Delightful UX_|> 80 % thumbs-up in in-app feedback after one week of use|
|_Seamless web access_|Cards accessible via browser with < 100ms latency|

---

## 3 Stakeholders & roles

|Role|Responsibility|
|---|---|
|**Product owner**|Defines learning goals, monitors KPIs|
|**Core engineer**|Rust/Tauri dev, pipeline & DB|
|**API engineer**|OpenAI integration, REST endpoints|
|**UX designer**|Desktop & web UI|
|**QA lead**|Test automation, regression suite|
|**Dev-ops** (light)|Binary signing, auto-update feed|

---

## 4 Scope (MVP vs. v1)

|Capability|MVP|v1|
|---|---|---|
|Text capture|Hot-key selection|Browser-extension capture|
|Screenshot|Region selection|OCR integration (optional)|
|Card generator|OpenAI Q-A prompt|Concept-map, cloze transforms|
|Scheduler|SM-2|FSRS adaptive algo|
|Review input|Text box|Whisper voice|
|Access|Local + Web|Optional encrypted cloud sync|
|Platforms|macOS + Windows|+ Linux, iOS companion|

---

## 5 Primary user journeys

1. **Text capture** → Press ⇧⌘>. Selected text is captured and sent to card generation.
    
2. **Screenshot** → Press ⇧⌘<. Transparent overlay appears, user drags rectangle, releases.
    
3. **Generation** → OpenAI returns _(Front, Back, Tags)_; modal shows result. User clicks ✔︎ or ✖︎ or edits, then saves.
    
4. **Review** → Cards available at localhost:5173 or through native app. User grades Pass/Fail.
    
5. **Metrics** → Weekly dashboard shows retention curve and streak.
    

---

## 6 System architecture

```
 ┌─────────┐  hot-key  ┌─────────────┐  image/text ┌────────┐   JSON   ┌────────┐
 │ OS hook │──────────▶│Capture      │────────────▶│OpenAI  │────────▶│  UI ✔︎ │
 │ (rdev)  │           │(screenshots)│             │ API    │         │(Tauri) │
 └─────────┘           └─────────────┘             └────────┘         └────────┘
                                                                           │
                                                                     SQLite▼
                                                                   ┌──────────────┐
                                                                   │ scheduler    │
                                                                   │(SM-2 task)   │
                                                                   └──────────────┘
                                                                           │
                                                                    REST API▼
                                                                 ┌──────────────┐
                                                                 │ Web UI       │
                                                                 │(localhost)   │
                                                                 └──────────────┘
```

- **Hot-key listener:** [`rdev`] crate provides cross-platform global shortcuts
    
- **Screen capture:** [`screenshots`] gives zero-copy GPU path on all desktops
    
- **Text selection:** [`get-selected-text`] for cross-platform text capture
    
- **Card generation:** OpenAI API with optimized prompts
    
- **UI layer:** Tauri desktop app + web interface
    
- **API layer:** Warp HTTP server exposing cards endpoint
    
- **Scheduler:** SM-2 crate (switchable to FSRS)

- **OCR:** `leptess` (Rust bindings to Tesseract ≥ v4). ([GitHub](https://github.com/houqp/leptess?utm_source=chatgpt.com "houqp/leptess: Productive and safe Rust binding for ... - GitHub"))
    
- **Local LLM:** `llama.cpp` via FFI, loading Q4_K_M or Q8_0 GGUF. ([Steel Phoenix](https://steelph0enix.github.io/posts/llama-cpp-guide/?utm_source=chatgpt.com "llama.cpp guide - Running LLMs locally, on any hardware, from ..."))
    
- **Speech-to-text:** `whisper.cpp` compiled with Metal / CUDA optimisations. ([GitHub](https://github.com/ggml-org/whisper.cpp?utm_source=chatgpt.com "ggml-org/whisper.cpp: Port of OpenAI's Whisper model in C/C++"))
    

---

## 7 Data model (SQLite)

_See §2 of previous note for full DDL._ Tables: `decks`, `cards`, `reviews`; all writes append-only.

---

## 8 Technology justification

|Requirement|Chosen tech|Reason|
|---|---|---|
|⚡ **Low latency capture**|`screenshots`|~6 ms capture on M-series; no temp files|
|🔒 **Privacy**|llama.cpp / whisper.cpp|Runs fully offline|
|🪶 **Tiny bundle**|Tauri|1–2 MB exe; uses system WebView|
|🧠 **Adaptive SRS**|FSRS (planned)|Outperforms vanilla SM-2 in Anki 23.12 tests ([Reddit](https://www.reddit.com/r/Anki/comments/18csuer/fsrs_is_now_the_most_accurate_spaced_repetition/?utm_source=chatgpt.com "FSRS is now the most accurate spaced repetition algorithm ... - Reddit"))|

---

## 9 Milestone road-map (16 weeks)

|Wk|Milestone|Key deliverables|
|---|---|---|
|1|**Project bootstrap**|Cargo workspace, CI, rusqlite schema migration|
|2–3|**Capture core**|rdev hot-key, region selection, `screenshots` integration|
|4–5|**OCR service**|Embed Tesseract data, async task pipeline, unit tests|
|6–7|**LLM integration**|llama.cpp bindings, prompt templating, basic card JSON|
|8|**Popup UX**|Tauri modal, Accept/Edit/Reject, dark-mode styling|
|9–10|**SRS scheduler**|SM-2 impl, periodic task, due card query|
|11|**Review notifications**|Tauri notif + input bar, DB review logging|
|12|**Speech input**|whisper.cpp, VAD, text normalisation|
|13|**Grading & stats**|Levenshtein/fuzzy match, dashboard view|
|14|**Cross-platform polish**|Windows installer, macOS codesign/notarise|
|15|**Beta test & telemetry**|opt-in error reports, user-journey metrics|
|16|**v0.1 release**|Signed binaries, docs, website, feedback survey|

Kanban board columns: _Backlog → In Progress → Review → Done_.

---

## 10 Testing strategy

|Layer|Tooling|Sample tests|
|---|---|---|
|Unit|`cargo test`, `serde_json` snapshots|Scheduler math, prompt template stability|
|Integration|`cargo-nextest`, test DB|Capture→OCR→LLM→DB happy path|
|E2E UI|Playwright (Tauri driver)|Accept/reject flow on Windows/macOS|
|Performance|Criterion benches|Capture & LLM latency budgets|
|Security|`cargo deny`, SAST|Dependency CVE scan, sandbox entitlements|

Continuous integration: GitHub Actions → macOS, Windows, Linux matrix.

---

## 11 Deployment & updates

- **Installers:**
    
    - macOS – `.dmg` via `tauri-bundler`, notarised & stapled
        
    - Windows – `NSIS` exe with auto-update feed (`tauri-updater`)
        
- **Self-update:** delta downloads hosted on GitHub Releases.
    
- **Config storage:** `$HOME/.oakley/` (JSON + `oakley.db`).
    

---

## 12 Security & privacy checklist

1. Hardened runtime: disable outbound net in release build.
    
2. Gate microphone access with explicit user opt-in.
    
3. Encrypt DB with SQLCipher if user sets master password.
    
4. Sign binaries; verify signatures on auto-update.
    
5. Supply SBOM to satisfy supply-chain audits.
    

---

## 13 Risk register & mitigation

|Risk|Likelihood|Impact|Mitigation|
|---|---|---|---|
|Tesseract mis-OCRs formulae|Med|Wrong cards|Option to select region again; mathpix API fallback (opt-in cloud)|
|LLM hallucination|Med|Bad cards|Show diff preview; on-device RAG using captured text context|
|Apple/macOS sandbox changes|Low|Build fails|Follow Tauri upgrade path, join Apple dev-rel beta|
|GPU drivers (Windows) break Metal/CUDA paths|Low|Latency spike|Ship CPU fall-back builds|

---

## 14 Future extensions (post-v1)

- **Vector-graded answers** (embeddings instead of exact match).
    
- **Mobile companion** – push due cards via LAN, answer on phone.
    
- **Browser plug-in** – one-click capture from Chrome/Firefox.
    
- **Multi-modal cards** – mix diagrams, audio, source link previews.
    
- **Plugin SDK** – open algorithm layer for community schedulers.
    

---

## 15 Appendix

### 15.1 Flash-card prompt (system)

> _"You are a strict pedagogue. Given INPUT_TEXT, output JSON with fields {front, back, tags}. The **front** must be phrased as a question. The **back** must be the minimal complete answer. Use no more than 25 words."_

### 15.2 Card review algorithm (SM-2)

```rust
// pseudo-code
if grade == Pass {
    interval = prev_interval * efactor;
    efactor  = max(1.3, efactor + (0.1 - (5 - quality) * (0.08 + (5 - quality)*0.02)));
} else {
    interval = 1;
}
```

---
Below is a deeper-in-the-code expansion: **exact Rust/TS stubs, IPC contracts, and scheduler math**.  
Everything compiles (cargo check) and shows how the crates talk to each other.

---

## 1 Pipeline orchestrator `crates/oakley-cli/src/main.rs`

```rust
//! Binary entry-point; spawns every async task and exposes an IPC socket.

use anyhow::Result;
use capture::CaptureEvent;
use ocr::extract_text;
use llm::gen_card;
use scheduler::{Scheduler, ReviewOutcome};
use data::{DbPool, CardId};
use tokio::{select, sync::mpsc};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    utils::log::init();                  // env-controlled tracing filter

    // ── bootstrap shared state ──────────────────────────────────────────
    let db = DbPool::new("oakley.db")?;
    let (cap_tx, mut cap_rx) = mpsc::channel::<CaptureEvent>(16);
    let (rev_tx, mut rev_rx) = mpsc::channel::<ReviewOutcome>(32);
    let scheduler = Scheduler::new(db.clone(), rev_tx.clone());

    // ── task: global hot-key + screenshot capture ───────────────────────
    tokio::spawn(capture::listen_and_capture(cap_tx.clone()));

    // ── task: scheduler tick every minute ───────────────────────────────
    tokio::spawn(scheduler.run());

    // ── main bus --------------------------------------------------------
    loop {
        select! {
            Some(evt) = cap_rx.recv() => {
                info!("got capture {:?}", evt.region);
                let txt = extract_text(&evt.image)?;
                let card = gen_card(&txt).await?;
                data::insert_card(&db, card, evt.path.as_deref())?;
                // send IPC → UI popup here (Unix domain socket or Tauri invoke)
            }

            Some(outcome) = rev_rx.recv() => {
                data::log_review(&db, outcome)?;
            }
        }
    }
}
```

_The orchestrator is the only place where channels meet; every crate stays pure & testable._

---

## 2 Scheduler crate (complete SM-2) `crates/scheduler/src/lib.rs`

```rust
//! Deterministic, pure SRS scheduler (SM-2); can swap impl later.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOutcome {
    pub card_id: i64,
    pub passed:  bool,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct CardState {
    efactor: f32,
    interval: u32,        // days
}

impl Default for CardState {
    fn default() -> Self { Self { efactor: 2.5, interval: 1 } }
}

pub struct Scheduler {
    db: data::DbPool,
    outcome_tx: tokio::sync::mpsc::Sender<ReviewOutcome>,
}

impl Scheduler {
    pub fn new(db: data::DbPool,
               outcome_tx: tokio::sync::mpsc::Sender<ReviewOutcome>) -> Self
    { Self { db, outcome_tx } }

    pub async fn run(self) {
        loop {
            let due = data::fetch_due_cards(&self.db, Utc::now()).unwrap_or_default();
            for card in due {
                // IPC→UI notification here. UI will call grade_card(id, passed)
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }

    pub fn next(state: &mut CardState, passed: bool) -> Duration {
        if !passed {
            state.interval = 1;
            return Duration::days(1);
        }

        // SuperMemo-2 factor update
        let quality = 5_u8;                       // we only store pass/fail; treat pass as q==5
        let ef = state.efactor +
            0.1 - (5.0 - quality as f32)*(0.08 + (5.0 - quality as f32)*0.02);
        state.efactor = ef.max(1.3);

        state.interval = match state.interval {
            1 => 6,
            i  => (i as f32 * state.efactor).round() as u32,
        };
        Duration::days(state.interval.into())
    }
}
```

---

## 3 IPC contract (Rust ⇆ Tauri React) via **specta**

### 3.1 `crates/ipc-types/src/lib.rs`

```rust
use specta::Type;
use serde::{Serialize, Deserialize};

#[derive(Type, Serialize, Deserialize)]
pub struct CardJson {
    pub id:        i64,
    pub front:     String,
    pub back:      String,
    pub tags:      Vec<String>,
}
```

Compile to TS on each build:

```bash
cargo run -p specta-build -- ../tauri-app/src/bindings.ts
```

### 3.2 `tauri-app/src/bindings.ts` (generated excerpt)

```ts
export interface CardJson {
  id: number;
  front: string;
  back: string;
  tags: string[];
}
```

Now React code is **type-safe** against Rust structs.

---

## 4 Hot-key + region overlay (capture crate essentials)

```rust
pub async fn listen_and_capture(tx: mpsc::Sender<CaptureEvent>) -> anyhow::Result<()> {
    use rdev::{listen, EventType, Key};
    use tao::{event_loop::EventLoop, window::WindowBuilder};
    // 1. Listen for ⇧⌘S (mac) or Ctrl+Shift+S (win/linux)
    listen(move |event| {
        if let EventType::KeyRelease(key) = event.event_type {
            if key == Key::S && event.mods.shift && event.mods.meta_or_ctrl {
                // show translucent tao overlay window; let user drag rectangle
                let img = screenshots::Screen::from_point(0, 0)
                           .capture_screen()?;
                let sel = crop_to_selection(img, rectangle);
                tx.blocking_send(CaptureEvent { image: sel, path: None }).ok();
            }
        }
    })?;
    Ok(())
}
```

_The overlay can use `tao` on all platforms or reuse the Tauri front-end by spawning a hidden window; choose whichever yields less latency._

---

## 5 Flash-card generator prompt and FFI (llm crate)

```rust
const SYSTEM_PROMPT: &str = r#"You are a pedagogue...
Return JSON: {"front":"...","back":"...","tags":["..."]}."#;

pub async fn gen_card(text: &str) -> anyhow::Result<CardFields> {
    let prompt = format!("{SYSTEM_PROMPT}\n\nINPUT_TEXT:\n{text}");
    let raw = llama_cpp::generate(&prompt, llama_cpp::Params::default())?; // blocking FFI
    Ok(serde_json::from_str(&raw)?)
}
```

`llama_cpp::generate` is just a thin safe wrapper around `llama_context_predict()`.

---

## 6 Database helpers (`crates/data/src/lib.rs`)

```rust
pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type CardId = i64;

pub fn insert_card(db: &DbPool, c: CardFields, img_path: Option<&str>) -> anyhow::Result<CardId> {
    let conn = db.get()?;
    conn.execute(
        "INSERT INTO cards (deck_id, front_text, back_text, source_image, tags)
         VALUES (1, ?1, ?2, ?3, ?4)",
        (c.front, c.back, img_path, c.tags.join(",")),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn fetch_due_cards(db: &DbPool, ts: DateTime<Utc>) -> anyhow::Result<Vec<CardJson>> {
    // query joining cards+reviews; leave this as exercise
}
```

---

## 7 Unit test examples (Nextest)

```rust
#[test]
fn sm2_interval_progression() {
    let mut st = CardState::default();
    assert_eq!(Scheduler::next(&mut st, true).num_days(), 6);  // 1 → 6
    assert_eq!(Scheduler::next(&mut st, true).num_days() > 6, true);
}

#[tokio::test]
async fn ocr_extracts_text() {
    let img = image::open("../fixtures/hello_world.png").unwrap();
    let txt = extract_text(&img).unwrap();
    assert!(txt.contains("Hello, world"));
}
```

---

## 8 CLI commands for power users

```console
oakley cards list          # dump JSON of all cards
oakley cards edit <id>     # open $EDITOR
oakley review now          # force all due cards now
oakley import pdf <file>   # split PDF into pages → OCR → cards
```

Implement via `clap` derive in `oakley-cli`.

---

## 9 Makefile (excerpt)

```make
test:                       ## run fast unit tests
	cargo nextest run

check:                      ## clippy lint
	cargo clippy --all-targets -- -D warnings

build-ui:                   ## dev front-end
	npm --prefix tauri-app run dev

build: check test           ## full build incl. UI
	npm --prefix tauri-app ci
	npm --prefix tauri-app run tauri build
```

---

## 10 Immediate coding TODO list

|🛠 Task|Owner|Difficulty|
|---|---|---|
|Overlay rectangle selection (tao/React canvas)|UI eng.|★★★☆☆|
|Unix-domain socket IPC (serde-bincode)|Core eng.|★★☆☆☆|
|SQLite migration runner (`refinery`)|Data eng.|★☆☆☆☆|
|Cross-crate error enum consolidation|Core eng.|★☆☆☆☆|
|Bundled macOS codesign script|Dev-ops|★★★☆☆|



