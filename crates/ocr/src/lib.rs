//! OCR service – mock implementation for development

use anyhow::Result;

// Mock leptess module
#[cfg(feature = "full")]
mod leptess {
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
    use anyhow::Result;

    pub fn extract_text(img: &image::RgbaImage) -> Result<String> {
        let mut tess = leptess::LepTess::new(None, "eng")?;
        tess.set_image_from_mem(img.as_raw())?;
        Ok(tess.get_utf8_text()?)
    }
}

#[cfg(not(feature = "full"))]
mod imp {
    use anyhow::Result;

    pub fn extract_text(_img: &image::RgbaImage) -> Result<String> {
        // Stub: return fixed string.
        Ok("stub OCR text".into())
    }
}

pub use imp::extract_text; 