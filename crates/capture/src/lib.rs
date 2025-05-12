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
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::path::PathBuf;
    use std::fs;

    /// Simple modifier state tracker
    #[derive(Default)]
    struct ModState {
        meta: bool,   // Command / Meta
        shift: bool,
    }

    /// Listen for hot-key (Cmd+Shift+<) and push capture events.
    pub async fn listen_and_capture(tx: Sender<CaptureEvent>) -> Result<()> {
        info!("🎯 Capture listener enabled, waiting for ⌘⇧<");

        // Channel to bridge between blocking hotkey thread and async world
        let (evt_tx, mut evt_rx) = tokio::sync::mpsc::channel::<()>(4);

        // Spawn blocking thread for rdev::listen (this call blocks)
        std::thread::spawn(move || {
            let state = Arc::new(Mutex::new(ModState::default()));
            let state_clone = state.clone();
            if let Err(e) = listen(move |event| {
                let mut st = state_clone.lock().unwrap();
                match event.event_type {
                    EventType::KeyPress(k) => {
                        info!("👆 Key press: {:?}", k);
                        match k {
                            Key::MetaLeft | Key::MetaRight => st.meta = true,
                            Key::ShiftLeft | Key::ShiftRight => st.shift = true,
                            Key::Comma => {
                                // '<' requires Shift on comma key
                                if st.meta && st.shift {
                                    info!("🔑 Hotkey detected: Command+Shift+<");
                                    let _ = evt_tx.blocking_send(());
                                }
                            }
                            _ => {}
                        }
                    },
                    EventType::KeyRelease(k) => {
                        info!("👇 Key release: {:?}", k);
                        match k {
                            Key::MetaLeft | Key::MetaRight => st.meta = false,
                            Key::ShiftLeft | Key::ShiftRight => st.shift = false,
                            _ => {}
                        }
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
                    info!("📸 Captured screen region: {}x{}", capture_event.region.2, capture_event.region.3);
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
        #[cfg(target_os = "macos")]
        {
            info!("Launching interactive screencapture utility (-i)");
            // Create a temporary PNG file with proper .png suffix – screencapture requires an extension
            let tmp = tempfile::Builder::new()
                .prefix("oakley_capture_")
                .suffix(".png")
                .tempfile()?;
            let path: PathBuf = tmp.path().into();

            // Run macOS screencapture with interactive selection (-i) and no sounds (-x)
            // -t png ensures PNG output
            let status = Command::new("screencapture")
                .args(["-x", "-i", "-t", "png", path.to_str().unwrap()])
                .status()?;

            if !status.success() {
                return Err(anyhow!("screencapture exited with status {status}"));
            }

            // If user cancelled, the file might be zero bytes
            let metadata = fs::metadata(&path)?;
            if metadata.len() == 0 {
                return Err(anyhow!("screenshot cancelled by user"));
            }

            let bytes = fs::read(&path)?;
            let dyn_img = image::load_from_memory(&bytes)?;
            let rgba = dyn_img.to_rgba8();
            let (w, h) = rgba.dimensions();

            info!("Interactive capture successful: {}x{}", w, h);

            return Ok(CaptureEvent {
                image: rgba,
                region: (0, 0, w, h), // selection region relative not known – default to full img size
                path: Some(path.to_string_lossy().to_string()),
            });
        }

        #[cfg(not(target_os = "macos"))]
        {
            info!("Attempting to capture full screen (non-macOS fallback)");
            // Capture the screen where cursor is, else first screen.
            let screen = Screen::all()
                .map_err(|e| anyhow!("screenshot list failed: {e}"))?
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("empty screen list"))?;
            info!("Detected screen: {}x{} at ({},{})", 
                  screen.display_info.width, screen.display_info.height,
                  screen.display_info.x, screen.display_info.y);
            let img = screen
                .capture()
                .map_err(|e| anyhow!("capture failed: {e}"))?;
            let (w, h) = (img.width(), img.height());
            info!("Captured image: {}x{}", w, h);
            let rgba = image::RgbaImage::from_raw(w, h, img.rgba().clone())
                .ok_or_else(|| anyhow!("buffer size mismatch"))?;
            info!("Converted to RGBA successfully");

            Ok(CaptureEvent {
                image: rgba,
                region: (0, 0, w as u32, h as u32),
                path: None,
            })
        }
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