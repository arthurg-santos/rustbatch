use gtk4 as gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use rustbatch_core::batch::BatchEvent;
use rustbatch_core::report::{BatchReport, JobReport, ReportStatus};
use crate::state::AppState;

pub fn spawn_batch_listener(
    rx: async_channel::Receiver<BatchEvent>,
    _list_store: gio::ListStore,
    state: Rc<RefCell<AppState>>,
    progress: gtk::ProgressBar,
    start_btn: gtk::Button,
    cancel_btn: gtk::Button,
) {
    let ctx = glib::MainContext::default();
    ctx.spawn_local(async move {
        let mut reports = Vec::new();
        let mut total_original = 0;
        let mut total_final = 0;

        while let Ok(event) = rx.recv().await {
            match event {
                BatchEvent::JobStarted(_) => {}
                BatchEvent::JobFinished(_, result) => {
                    match result {
                        Ok(res) => {
                            total_original += res.original_size;
                            total_final += res.final_size;
                            reports.push(JobReport {
                                source: res.output_path.clone(), 
                                output: Some(res.output_path),
                                status: ReportStatus::Success,
                                original_size: res.original_size,
                                final_size: Some(res.final_size),
                                error: None,
                            });
                        }
                        Err(e) => {
                            reports.push(JobReport {
                                source: "desconhecido".into(),
                                output: None,
                                status: ReportStatus::Failed,
                                original_size: 0,
                                final_size: None,
                                error: Some(e.to_string()),
                            });
                        }
                    }
                }
                BatchEvent::Progress(done, total) => {
                    let fraction = done as f64 / total as f64;
                    progress.set_fraction(fraction);
                    progress.set_text(Some(&format!("{done} de {total} concluídos")));
                }
                BatchEvent::Finished => {
                    start_btn.set_sensitive(true);
                    cancel_btn.set_sensitive(false);
                    state.borrow_mut().cancel.store(false, std::sync::atomic::Ordering::Relaxed);
                    
                    let batch_report = BatchReport {
                        jobs: std::mem::take(&mut reports),
                        total_original,
                        total_final,
                    };
                    
                    // CORREÇÃO: AlertDialog não tem .title(), e .show() espera um Option<&Window>
                    let dialog = gtk::AlertDialog::builder()
                        .message("Processamento concluído")
                        .detail(format!(
                            "Economia de espaço: {:.1}%\nTamanho original: {:.2} MB\nTamanho final: {:.2} MB",
                            batch_report.savings_percent(),
                            total_original as f64 / 1_048_576.0,
                            total_final as f64 / 1_048_576.0
                        ))
                        .build();
                    
                    dialog.show(None::<&gtk::Window>); 
                }
            }
        }
    });
}