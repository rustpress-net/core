//! Image editing functionality
//!
//! Provides image editing capabilities including:
//! - Crop
//! - Resize
//! - Rotate/Flip
//! - Filters and adjustments

use image::{DynamicImage, GenericImageView, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::{MediaError, MediaResult};

/// Image editor
pub struct ImageEditor {
    image: DynamicImage,
}

impl ImageEditor {
    /// Load image from bytes
    pub fn from_bytes(data: &[u8]) -> MediaResult<Self> {
        let image = image::load_from_memory(data)?;
        Ok(Self { image })
    }

    /// Load image from file
    pub fn from_file(path: &str) -> MediaResult<Self> {
        let image = image::open(path)?;
        Ok(Self { image })
    }

    /// Get current dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    /// Crop image
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> &mut Self {
        self.image = self.image.crop_imm(x, y, width, height);
        self
    }

    /// Crop to aspect ratio (centered)
    pub fn crop_to_aspect(&mut self, ratio: f64) -> &mut Self {
        let (w, h) = self.dimensions();
        let current_ratio = w as f64 / h as f64;

        let (crop_w, crop_h) = if current_ratio > ratio {
            // Image is wider, crop width
            ((h as f64 * ratio) as u32, h)
        } else {
            // Image is taller, crop height
            (w, (w as f64 / ratio) as u32)
        };

        let x = (w - crop_w) / 2;
        let y = (h - crop_h) / 2;

        self.crop(x, y, crop_w, crop_h)
    }

    /// Resize image maintaining aspect ratio
    pub fn resize(&mut self, max_width: u32, max_height: u32) -> &mut Self {
        self.image = self.image.resize(
            max_width,
            max_height,
            image::imageops::FilterType::Lanczos3,
        );
        self
    }

    /// Resize to exact dimensions
    pub fn resize_exact(&mut self, width: u32, height: u32) -> &mut Self {
        self.image = self.image.resize_exact(
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );
        self
    }

    /// Scale by percentage
    pub fn scale(&mut self, percent: f32) -> &mut Self {
        let (w, h) = self.dimensions();
        let new_w = (w as f32 * percent / 100.0) as u32;
        let new_h = (h as f32 * percent / 100.0) as u32;
        self.resize_exact(new_w.max(1), new_h.max(1))
    }

    /// Rotate 90 degrees clockwise
    pub fn rotate_90(&mut self) -> &mut Self {
        self.image = self.image.rotate90();
        self
    }

    /// Rotate 180 degrees
    pub fn rotate_180(&mut self) -> &mut Self {
        self.image = self.image.rotate180();
        self
    }

    /// Rotate 270 degrees (90 counter-clockwise)
    pub fn rotate_270(&mut self) -> &mut Self {
        self.image = self.image.rotate270();
        self
    }

    /// Flip horizontally
    pub fn flip_horizontal(&mut self) -> &mut Self {
        self.image = self.image.fliph();
        self
    }

    /// Flip vertically
    pub fn flip_vertical(&mut self) -> &mut Self {
        self.image = self.image.flipv();
        self
    }

    /// Convert to grayscale
    pub fn grayscale(&mut self) -> &mut Self {
        self.image = self.image.grayscale();
        self
    }

    /// Invert colors
    pub fn invert(&mut self) -> &mut Self {
        self.image.invert();
        self
    }

    /// Adjust brightness (-100 to 100)
    pub fn brightness(&mut self, value: i32) -> &mut Self {
        let value = value.clamp(-100, 100);
        self.image = self.image.brighten(value);
        self
    }

    /// Adjust contrast (factor, 1.0 = no change)
    pub fn contrast(&mut self, factor: f32) -> &mut Self {
        let factor = factor.clamp(0.0, 10.0);
        self.image = self.image.adjust_contrast(factor);
        self
    }

    /// Apply Gaussian blur
    pub fn blur(&mut self, sigma: f32) -> &mut Self {
        self.image = self.image.blur(sigma);
        self
    }

    /// Apply unsharpen mask
    pub fn sharpen(&mut self, sigma: f32, threshold: i32) -> &mut Self {
        self.image = self.image.unsharpen(sigma, threshold);
        self
    }

    /// Adjust hue rotation (in degrees, 0-360)
    pub fn hue_rotate(&mut self, degrees: i32) -> &mut Self {
        self.image = self.image.huerotate(degrees);
        self
    }

    /// Apply sepia filter
    pub fn sepia(&mut self) -> &mut Self {
        let mut rgba = self.image.to_rgba8();

        for pixel in rgba.pixels_mut() {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            pixel[0] = ((r * 0.393) + (g * 0.769) + (b * 0.189)).min(255.0) as u8;
            pixel[1] = ((r * 0.349) + (g * 0.686) + (b * 0.168)).min(255.0) as u8;
            pixel[2] = ((r * 0.272) + (g * 0.534) + (b * 0.131)).min(255.0) as u8;
        }

        self.image = DynamicImage::ImageRgba8(rgba);
        self
    }

    /// Apply vignette effect
    pub fn vignette(&mut self, strength: f32) -> &mut Self {
        let mut rgba = self.image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for (x, y, pixel) in rgba.enumerate_pixels_mut() {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;

            let factor = 1.0 - (dist * strength).min(1.0);

            pixel[0] = (pixel[0] as f32 * factor) as u8;
            pixel[1] = (pixel[1] as f32 * factor) as u8;
            pixel[2] = (pixel[2] as f32 * factor) as u8;
        }

        self.image = DynamicImage::ImageRgba8(rgba);
        self
    }

    /// Apply preset filter
    pub fn apply_preset(&mut self, preset: FilterPreset) -> &mut Self {
        match preset {
            FilterPreset::Original => self,
            FilterPreset::Vintage => {
                self.contrast(0.9)
                    .sepia()
                    .brightness(-10)
            }
            FilterPreset::Dramatic => {
                self.contrast(1.4)
                    .brightness(-20)
                    .vignette(0.5)
            }
            FilterPreset::BW => {
                self.grayscale()
                    .contrast(1.1)
            }
            FilterPreset::Warm => {
                self.hue_rotate(15)
                    .brightness(10)
            }
            FilterPreset::Cool => {
                self.hue_rotate(-15)
                    .brightness(5)
            }
            FilterPreset::Vivid => {
                self.contrast(1.3)
                    .brightness(10)
            }
            FilterPreset::Muted => {
                self.contrast(0.8)
                    .brightness(5)
            }
        }
    }

    /// Apply watermark
    pub fn watermark(&mut self, watermark_data: &[u8], position: WatermarkPosition, opacity: f32) -> MediaResult<&mut Self> {
        let watermark = image::load_from_memory(watermark_data)?;
        let (img_w, img_h) = self.dimensions();
        let (wm_w, wm_h) = watermark.dimensions();

        // Calculate position
        let (x, y) = match position {
            WatermarkPosition::TopLeft => (10, 10),
            WatermarkPosition::TopRight => ((img_w - wm_w).saturating_sub(10), 10),
            WatermarkPosition::BottomLeft => (10, (img_h - wm_h).saturating_sub(10)),
            WatermarkPosition::BottomRight => (
                (img_w - wm_w).saturating_sub(10),
                (img_h - wm_h).saturating_sub(10),
            ),
            WatermarkPosition::Center => (
                (img_w - wm_w) / 2,
                (img_h - wm_h) / 2,
            ),
            WatermarkPosition::Custom(cx, cy) => (cx, cy),
        };

        // Overlay watermark
        let mut rgba = self.image.to_rgba8();
        let wm_rgba = watermark.to_rgba8();

        for (wx, wy, wm_pixel) in wm_rgba.enumerate_pixels() {
            let px = x + wx;
            let py = y + wy;

            if px < img_w && py < img_h {
                let img_pixel = rgba.get_pixel_mut(px, py);
                let alpha = (wm_pixel[3] as f32 / 255.0) * opacity;

                for i in 0..3 {
                    img_pixel[i] = (
                        (1.0 - alpha) * img_pixel[i] as f32 +
                        alpha * wm_pixel[i] as f32
                    ) as u8;
                }
            }
        }

        self.image = DynamicImage::ImageRgba8(rgba);
        Ok(self)
    }

    /// Export to bytes
    pub fn to_bytes(&self, format: ImageFormat) -> MediaResult<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        self.image.write_to(&mut buffer, format)?;
        Ok(buffer.into_inner())
    }

    /// Export to JPEG with quality
    pub fn to_jpeg(&self, quality: u8) -> MediaResult<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
        self.image.write_with_encoder(encoder)?;
        Ok(buffer.into_inner())
    }

    /// Export to PNG
    pub fn to_png(&self) -> MediaResult<Vec<u8>> {
        self.to_bytes(ImageFormat::Png)
    }

    /// Export to WebP
    pub fn to_webp(&self) -> MediaResult<Vec<u8>> {
        self.to_bytes(ImageFormat::WebP)
    }

    /// Save to file
    pub fn save(&self, path: &str) -> MediaResult<()> {
        self.image.save(path)?;
        Ok(())
    }
}

/// Filter presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterPreset {
    Original,
    Vintage,
    Dramatic,
    BW,
    Warm,
    Cool,
    Vivid,
    Muted,
}

/// Watermark position
#[derive(Debug, Clone, Copy)]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom(u32, u32),
}

/// Edit operations for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditOperation {
    Crop { x: u32, y: u32, width: u32, height: u32 },
    CropToAspect { ratio: f64 },
    Resize { width: u32, height: u32 },
    ResizeExact { width: u32, height: u32 },
    Scale { percent: f32 },
    Rotate90,
    Rotate180,
    Rotate270,
    FlipHorizontal,
    FlipVertical,
    Grayscale,
    Invert,
    Brightness { value: i32 },
    Contrast { factor: f32 },
    Blur { sigma: f32 },
    Sharpen { sigma: f32, threshold: i32 },
    HueRotate { degrees: i32 },
    Sepia,
    Vignette { strength: f32 },
    Preset { preset: FilterPreset },
}

impl EditOperation {
    /// Apply operation to editor
    pub fn apply(&self, editor: &mut ImageEditor) {
        match self {
            EditOperation::Crop { x, y, width, height } => {
                editor.crop(*x, *y, *width, *height);
            }
            EditOperation::CropToAspect { ratio } => {
                editor.crop_to_aspect(*ratio);
            }
            EditOperation::Resize { width, height } => {
                editor.resize(*width, *height);
            }
            EditOperation::ResizeExact { width, height } => {
                editor.resize_exact(*width, *height);
            }
            EditOperation::Scale { percent } => {
                editor.scale(*percent);
            }
            EditOperation::Rotate90 => {
                editor.rotate_90();
            }
            EditOperation::Rotate180 => {
                editor.rotate_180();
            }
            EditOperation::Rotate270 => {
                editor.rotate_270();
            }
            EditOperation::FlipHorizontal => {
                editor.flip_horizontal();
            }
            EditOperation::FlipVertical => {
                editor.flip_vertical();
            }
            EditOperation::Grayscale => {
                editor.grayscale();
            }
            EditOperation::Invert => {
                editor.invert();
            }
            EditOperation::Brightness { value } => {
                editor.brightness(*value);
            }
            EditOperation::Contrast { factor } => {
                editor.contrast(*factor);
            }
            EditOperation::Blur { sigma } => {
                editor.blur(*sigma);
            }
            EditOperation::Sharpen { sigma, threshold } => {
                editor.sharpen(*sigma, *threshold);
            }
            EditOperation::HueRotate { degrees } => {
                editor.hue_rotate(*degrees);
            }
            EditOperation::Sepia => {
                editor.sepia();
            }
            EditOperation::Vignette { strength } => {
                editor.vignette(*strength);
            }
            EditOperation::Preset { preset } => {
                editor.apply_preset(*preset);
            }
        }
    }
}

/// Apply multiple operations
pub fn apply_operations(data: &[u8], operations: &[EditOperation]) -> MediaResult<Vec<u8>> {
    let mut editor = ImageEditor::from_bytes(data)?;

    for op in operations {
        op.apply(&mut editor);
    }

    editor.to_jpeg(85)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require actual image data
    #[test]
    fn test_filter_preset_serialization() {
        let preset = FilterPreset::Vintage;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"vintage\"");
    }

    #[test]
    fn test_edit_operation_serialization() {
        let op = EditOperation::Brightness { value: 10 };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("brightness"));
        assert!(json.contains("10"));
    }
}
