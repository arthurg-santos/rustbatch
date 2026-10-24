use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use crate::error::CoreError;
use crate::model::{CollisionPolicy, OutputConfig};

pub fn resolve_output_path(
    source: &Path,
    desired_name: &str,
    cfg: &OutputConfig,
) -> Result<PathBuf, CoreError> {
    let mut dir = cfg.directory.clone();
    if cfg.create_subfolder {
        dir = dir.join("rustbatch-out");
    }

    if cfg.preserve_structure {
        if let Some(parent) = source.parent() {
            // Mantém apenas o nome da pasta de origem, não o caminho absoluto.
            if let Some(folder_name) = parent.file_name() {
                dir = dir.join(folder_name);
            }
        }
    }

    fs::create_dir_all(&dir).map_err(|e| CoreError::io(&dir, e))?;

    let mut candidate = dir.join(desired_name);

    match cfg.collision {
        CollisionPolicy::Unique => {
            let mut n = 1u32;
            while candidate.exists() {
                let stem = Path::new(desired_name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("imagem");
                let ext = Path::new(desired_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("bin");
                candidate = dir.join(format!("{stem}_{n}.{ext}"));
                n += 1;
                if n > 10_000 {
                    return Err(CoreError::NameConflict(candidate));
                }
            }
        }
        CollisionPolicy::Overwrite => { /* permitido, mas a UI deve confirmar */ }
        CollisionPolicy::Skip => {
            if candidate.exists() {
                return Err(CoreError::NameConflict(candidate));
            }
        }
    }

    Ok(candidate)
}

pub fn write_atomic(
    final_path: &Path,
    write_fn: impl FnOnce(&mut fs::File) -> Result<(), CoreError>,
) -> Result<(), CoreError> {
    let dir = final_path.parent()
        .ok_or_else(|| CoreError::InvalidOutputPath(final_path.to_path_buf()))?;

    let mut tmp = NamedTempFile::new_in(dir)
        .map_err(|e| CoreError::io(dir, e))?;

    write_fn(tmp.as_file_mut())?;
    tmp.as_file_mut().sync_all()
        .map_err(|e| CoreError::io(tmp.path(), e))?;

    tmp.persist(final_path)
        .map_err(|e| CoreError::io(final_path, e.error))?;

    Ok(())
}