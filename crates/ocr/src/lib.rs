//! OCR service – mock implementation for development

use anyhow::Result;
use tracing::info;

// Mock Tesseract when external crate is unavailable
#[cfg(all(feature = "full", not(feature = "use-crate")))]
mod leptess_mock {
    pub struct LepTess;
    
    impl LepTess {
        pub fn new(_datapath: Option<&str>, _lang: &str) -> anyhow::Result<Self> {
            Ok(Self {})
        }
        
        pub fn set_image_from_mem(&mut self, _img_data: &[u8]) -> anyhow::Result<()> {
            Ok(())
        }
        
        pub fn get_utf8_text(&self) -> anyhow::Result<String> {
            Ok("Mock OCR text from Tesseract".to_string())
        }
    }
}

#[cfg(feature = "full")]
mod imp {
    use super::*;
    use leptess::LepTess;
    use anyhow::Result;

    pub fn extract_text(img: &image::RgbaImage) -> Result<String> {
        info!("OCR: Extracting text from {}x{} image", img.width(), img.height());

        // Initialize Tesseract. If TESSDATA_PREFIX is set in env, LepTess will pick it up.
        let mut tess = LepTess::new(None, "eng")?;
        tess.set_image_from_mem(img.as_raw())?;
        let text = tess.get_utf8_text()?;
        let text_trimmed = text.trim().to_string();
        info!("OCR: Extracted {} characters of text", text_trimmed.len());
        Ok(text_trimmed)
    }
}

#[cfg(not(feature = "full"))]
mod imp {
    use anyhow::Result;
    use tracing::info;

    pub fn extract_text(_img: &image::RgbaImage) -> Result<String> {
        // In stub mode, we can add some debug data about the image
        let width = _img.width();
        let height = _img.height();
        info!("OCR STUB: Received {}x{} image for text extraction (stub mode)", width, height);
        // Stub: return fixed string.
        let stub_text = "stub OCR text for testing with more content to process";
        info!("OCR STUB: Returning stub text: '{}'", stub_text);
        Ok(stub_text.into())
    }
}

pub use imp::extract_text; 