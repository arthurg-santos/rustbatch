use std::path::PathBuf;
use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Processing,
    Done,
    Skipped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SourceMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct JobResult {
    pub output_path: PathBuf,
    pub original_size: u64,
    pub final_size: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct ImageJob {
    pub source: PathBuf,
    pub metadata: Option<SourceMetadata>,
    pub status: JobStatus,
    pub result: Option<JobResult>,
    pub error: Option<CoreError>,
}

impl ImageJob {
    pub fn new(source: PathBuf) -> Self {
        Self {
            source,
            metadata: None,
            status: JobStatus::Pending,
            result: None,
            error: None,
        }
    }
}