use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::error::CoreError;
use crate::model::RenameConfig;

pub fn build_output_name(
    source: &Path,
    cfg: &RenameConfig,
    index: u32,
    extension: &str,
) -> Result<String, CoreError> {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CoreError::InvalidOutputPath(source.to_path_buf()))?;

    // Substituição de texto no nome
    let mut name = if cfg.replace_from.is_empty() {
        stem.to_string()
    } else {
        stem.replace(&cfg.replace_from, &cfg.replace_to)
    };

    // Prefixo e sufixo
    name = format!("{}{}{}", cfg.prefix, name, cfg.suffix);

    // Data (AAAA-MM-DD)
    if cfg.include_date {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        name = format!("{name}_{date}");
    }

    // Numeração sequencial com zeros à esquerda
    if cfg.index_padding > 0 {
        let padded = format!(
            "{:0width$}",
            index,
            width = cfg.index_padding as usize
        );
        name = format!("{name}_{padded}");
    }

    // Sanitização contra path traversal
    let sanitized = sanitize_name(&name)?;
    Ok(format!("{}.{}", sanitized, extension))
}

fn sanitize_name(name: &str) -> Result<String, CoreError> {
    if name.contains("..") || name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(CoreError::InvalidOutputPath(PathBuf::from(name)));
    }
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CoreError::InvalidOutputPath(PathBuf::from(name)));
    }
    Ok(trimmed.to_string())
}

pub fn detect_conflicts(
    sources: &[PathBuf],
    cfg: &RenameConfig,
    extension: &str,
    output_dir: &Path,
) -> Result<(), Vec<(PathBuf, PathBuf)>> {
    let mut seen = HashSet::new();
    let mut conflicts = Vec::new();

    for (i, source) in sources.iter().enumerate() {
        let idx = cfg.start_index + i as u32;
        let Ok(name) = build_output_name(source, cfg, idx, extension) else {
            continue;
        };
        let full = output_dir.join(&name);

        if !seen.insert(full.clone()) {
            conflicts.push((source.clone(), full));
        }
    }

    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(conflicts)
    }
}