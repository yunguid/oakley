//! Stub of capture subsystem. When built with `--features full`, integrates
//! global hot-key listener and region capture; otherwise, exposes no-op impls
//! for unit testing.

/// A simple wrapper around image::RgbaImage that doesn't need to be serialized
#[derive(Debug, Clone)]
pub struct CaptureEvent {
    pub image: image::RgbaImage,
    pub region: (u32, u32, u32, u32), // x,y,w,h
    pub path: Option<String>,         // optional disk location
}

#[cfg(feature = "full")]
mod imp {
    use super::*;
    use tokio::sync::mpsc::Sender;
    use tracing::{info, error};
    use rdev::{listen, EventType, Key};
    use screenshots::Screen;
    use std::sync::{Arc, Mutex};
    use anyhow::{Result, anyhow};

    /// Simple modifier state tracker
    #[derive(Default)]
    struct ModState {
        meta: bool,   // Command / Meta
        shift: bool,
    }

    /// Listen for hot-key (Cmd+Shift+,) and push capture events.
    pub async fn listen_and_capture(tx: Sender<CaptureEvent>) -> Result<()> {
        info!("capture listener enabled");

        // Channel to bridge between blocking hotkey thread and async world
        let (evt_tx, mut evt_rx) = tokio::sync::mpsc::channel::<()>(4);

        // Spawn blocking thread for rdev::listen (this call blocks)
        std::thread::spawn(move || {
            let state = Arc::new(Mutex::new(ModState::default()));
            let state_clone = state.clone();
            if let Err(e) = listen(move |event| {
                let mut st = state_clone.lock().unwrap();
                match event.event_type {
                    EventType::KeyPress(k) => match k {
                        Key::MetaLeft | Key::MetaRight => st.meta = true,
                        Key::ShiftLeft | Key::ShiftRight => st.shift = true,
                        Key::Comma | Key::Dot => {
                            if st.meta && st.shift {
                                // Hotkey detected
                                let _ = evt_tx.blocking_send(());
                            }
                        }
                        _ => {}
                    },
                    EventType::KeyRelease(k) => match k {
                        Key::MetaLeft | Key::MetaRight => st.meta = false,
                        Key::ShiftLeft | Key::ShiftRight => st.shift = false,
                        _ => {}
                    },
                    _ => {}
                }
            }) {
                error!(?e, "rdev listen error");
            }
        });

        // Async loop waiting for trigger events
        while evt_rx.recv().await.is_some() {
            match capture_screen() {
                Ok(capture_event) => {
                    if tx.send(capture_event).await.is_err() {
                        error!("Main receiver dropped; stopping capture listener");
                        break;
                    }
                }
                Err(e) => {
                    error!(?e, "failed to capture screen");
                }
            }
        }

        Ok(())
    }

    fn capture_screen() -> Result<CaptureEvent> {
        // Capture the screen where cursor is, else first screen.
        let screen = Screen::all()
            .map_err(|e| anyhow!("screenshot list failed: {e}"))?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("empty screen list"))?;
        let img = screen
            .capture()
            .map_err(|e| anyhow!("capture failed: {e}"))?;
        let (w, h) = (img.width(), img.height());
        let mut buf = img.buffer(); // BGRA
        // Convert BGRA -> RGBA in place
        for chunk in buf.chunks_mut(4) {
            chunk.swap(0, 2); // swap B and R
        }
        let rgba = image::RgbaImage::from_raw(w, h, buf)
            .ok_or_else(|| anyhow!("buffer size mismatch"))?;

        Ok(CaptureEvent {
            image: rgba,
            region: (0, 0, w as u32, h as u32),
            path: None,
        })
    }
}

#[cfg(not(feature = "full"))]
mod imp {
    use super::*;
    use tokio::sync::mpsc::Sender;
    use anyhow::Result;

    pub async fn listen_and_capture(_tx: Sender<CaptureEvent>) -> Result<()> {
        // No-op in stub builds.
        Ok(())
    }
}

pub use imp::listen_and_capture; 