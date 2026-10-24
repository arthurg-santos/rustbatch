use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use rustbatch_core::model::{ProcessConfig, ImageJob, OutputFormat, CompressionConfig, RenameConfig, OutputConfig, CollisionPolicy, Rotation};

#[derive(Clone, Debug)]
pub struct UiJob {
    pub name: String, pub width: u32, pub height: u32, pub path: PathBuf,
}

impl UiJob {
    pub fn to_image_job(&self) -> ImageJob { ImageJob::new(self.path.clone()) }
}

pub struct AppState {
    pub jobs: Vec<UiJob>,
    pub config: ProcessConfig,
    pub cancel: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            config: default_config(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn default_config() -> ProcessConfig {
    ProcessConfig {
        resize: None,
        output_format: OutputFormat::Jpeg,
        compression: CompressionConfig { quality: Some(85), png_compression: None },
        filters: Vec::new(),
        rotation: Rotation::None,
        watermark: None,
        rename: RenameConfig::default(),
        output: OutputConfig {
            directory: std::env::temp_dir().join("rustbatch-out"),
            collision: CollisionPolicy::Unique,
            create_subfolder: false,
            preserve_structure: false,
        },
    }
}