use std::path::{Path, PathBuf};
use image::ImageReader;
use crate::error::CoreError;
use crate::model::SourceMetadata;

pub fn inspect_file(path: &Path) -> Result<SourceMetadata, CoreError> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| CoreError::io(path, e))?;

    let reader = ImageReader::open(path)
        .map_err(|e| CoreError::io(path, e))?;

    let reader = reader.with_guessed_format()
        .map_err(|e| CoreError::io(path, e))?;

    let format = reader.format()
        .map(|f| format!("{:?}", f))
        .unwrap_or_else(|| "Desconhecido".to_string());

    let (width, height) = reader.into_dimensions()
        .map_err(|e| CoreError::CorruptFile {
            path: path.to_path_buf(),
            source: e,
        })?;

    Ok(SourceMetadata {
        width,
        height,
        format,
        size_bytes: metadata.len(),
    })
}

pub fn discover_in_dir(
    dir: &Path,
    recursive: bool,
) -> Result<Vec<PathBuf>, CoreError> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current)
            .map_err(|e| CoreError::io(&current, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| CoreError::io(&current, e))?;
            let path = entry.path();

            if path.is_dir() {
                if recursive {
                    stack.push(path);
                }
            } else if is_supported_extension(&path) {
                out.push(path);
            }
        }
    }

    out.sort();
    Ok(out)
}

fn is_supported_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp" | "avif" | "tif" | "tiff" | "bmp" | "gif")
    )
}