use std::num::NonZeroU32;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Jpeg,
    Png,
    WebP { lossless: bool },
    Avif,
    Tiff,
    Bmp,
    Gif,
    Heic,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP { .. } => "webp",
            Self::Avif => "avif",
            Self::Tiff => "tif",
            Self::Bmp => "bmp",
            Self::Gif => "gif",
            Self::Heic => "heic",
        }
    }

    pub fn supports_alpha(&self) -> bool {
        matches!(self, Self::Png | Self::WebP { .. } | Self::Avif | Self::Tiff | Self::Gif)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResizeMode {
    Exact { width: NonZeroU32, height: NonZeroU32 },
    Percentage(f32),
    FitWithin { width: NonZeroU32, height: NonZeroU32 },
}

#[derive(Debug, Clone)]
pub struct ResizeConfig {
    pub mode: ResizeMode,
    pub preserve_aspect: bool,
    pub no_upscale: bool,
}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// 1–100 para JPEG/AVIF; 0–9 para PNG.
    pub quality: Option<u8>,
    /// Nível de compressão para PNG (0–9).
    pub png_compression: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    Brightness(i32),   // -100..=100
    Contrast(f32),     // 0.0..=3.0
    Saturation(f32),   // 0.0..=3.0
    Grayscale,
    Sepia,
    Blur(f32),         // sigma
    Sharpen,
    Invert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    R90,
    R180,
    R270,
    FlipH,
    FlipV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkPosition {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, Center, MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
}

#[derive(Debug, Clone)]
pub enum Watermark {
    Text {
        text: String,
        font_size: f32,
        color: [u8; 4],
        opacity: f32,
        margin: u32,
        position: WatermarkPosition,
    },
    Image {
        path: PathBuf,
        scale: f32,
        opacity: f32,
        margin: u32,
        position: WatermarkPosition,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionPolicy {
    Unique,
    Overwrite,
    Skip,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub directory: PathBuf,
    pub collision: CollisionPolicy,
    pub create_subfolder: bool,
    pub preserve_structure: bool,
}

#[derive(Debug, Clone)]
pub struct RenameConfig {
    pub prefix: String,
    pub suffix: String,
    pub replace_from: String,
    pub replace_to: String,
    pub start_index: u32,
    pub index_padding: u8,
    pub include_date: bool,
}

impl Default for RenameConfig {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            replace_from: String::new(),
            replace_to: String::new(),
            start_index: 1,
            index_padding: 4,
            include_date: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub resize: Option<ResizeConfig>,
    pub output_format: OutputFormat,
    pub compression: CompressionConfig,
    pub filters: Vec<Filter>,
    pub rotation: Rotation,
    pub watermark: Option<Watermark>,
    pub rename: RenameConfig,
    pub output: OutputConfig,
}

pub const MAX_DIMENSION: u32 = 20_000;
pub const MAX_PIXELS: u64 = 100_000_000;