use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use rayon::prelude::*;
use crate::error::CoreError;
use crate::model::*;
use crate::pipeline::process_image;

#[derive(Debug)]
pub enum BatchEvent {
    JobStarted(usize),
    JobFinished(usize, Result<JobResult, CoreError>),
    Progress(usize, usize), // (concluídos, total)
    Finished,
}

pub fn run_batch(
    jobs: Vec<ImageJob>,
    cfg: ProcessConfig,
    cancel: Arc<AtomicBool>,
    events: async_channel::Sender<BatchEvent>,
) {
    let total = jobs.len();
    let done = Arc::new(AtomicUsize::new(0));

    jobs.into_par_iter().enumerate().for_each(|(idx, job)| {
        if cancel.load(Ordering::Relaxed) {
            let _ = events.send_blocking(BatchEvent::JobFinished(
                idx,
                Err(CoreError::Cancelled),
            ));
            return;
        }

        let _ = events.send_blocking(BatchEvent::JobStarted(idx));

        let result = process_one(&job, &cfg);

        let _ = events.send_blocking(BatchEvent::JobFinished(idx, result));

        let completed = done.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = events.send_blocking(BatchEvent::Progress(completed, total));
    });

    let _ = events.send_blocking(BatchEvent::Finished);
}

fn process_one(job: &ImageJob, cfg: &ProcessConfig) -> Result<JobResult, CoreError> {
    let original_size = std::fs::metadata(&job.source)
        .map_err(|e| CoreError::io(&job.source, e))?
        .len();

    let img = process_image(&job.source, cfg)?;

    // Renomeação e destino serão refinados no próximo passo.
    let file_name = job.source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("imagem");

    let out_path = crate::output::resolve_output_path(
        &job.source,
        file_name,
        &cfg.output,
    )?;

    crate::output::write_atomic(&out_path, |file| {
        img.write_to(
            &mut std::io::BufWriter::new(file),
            image::ImageFormat::from_extension(cfg.output_format.extension())
                .ok_or_else(|| CoreError::UnsupportedFormat(
                    cfg.output_format.extension().to_string(),
                ))?,
        )
        .map_err(|e| CoreError::Encode(e.to_string()))
    })?;

    let final_size = std::fs::metadata(&out_path)
        .map_err(|e| CoreError::io(&out_path, e))?
        .len();

    Ok(JobResult {
        output_path: out_path,
        original_size,
        final_size,
        width: img.width(),
        height: img.height(),
    })
}