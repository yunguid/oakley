//! Stub of capture subsystem. When built with `--features full`, integrates
//! global hot-key listener and region capture; otherwise, exposes no-op impls
//! for unit testing.

use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    use tracing::info;

    /// Listen for hot-key and push capture events.
    pub async fn listen_and_capture(tx: Sender<CaptureEvent>) -> Result<()> {
        info!("capture listener enabled (stub)");
        // TODO: implement rdev & screenshots integration.
        Ok(())
    }
}

#[cfg(not(feature = "full"))]
mod imp {
    use super::*;
    use tokio::sync::mpsc::Sender;

    pub async fn listen_and_capture(_tx: Sender<CaptureEvent>) -> Result<()> {
        // No-op in stub builds.
        Ok(())
    }
}

pub use imp::listen_and_capture; 