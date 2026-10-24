use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JobReport {
    pub source: PathBuf,
    pub output: Option<PathBuf>,
    pub status: ReportStatus,
    pub original_size: u64,
    pub final_size: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReportStatus {
    Success,
    Skipped,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchReport {
    pub jobs: Vec<JobReport>,
    pub total_original: u64,
    pub total_final: u64,
}

impl BatchReport {
    pub fn savings_percent(&self) -> f64 {
        if self.total_original == 0 {
            return 0.0;
        }
        let saved = self.total_original as f64 - self.total_final as f64;
        (saved / self.total_original as f64) * 100.0
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_csv(&self) -> String {
        let mut out = String::from("arquivo,saida,status,tamanho_original,tamanho_final,erro\n");
        for j in &self.jobs {
            out.push_str(&format!(
                "{},{},{:?},{},{},{}\n",
                j.source.display(),
                j.output.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
                j.status,
                j.original_size,
                j.final_size.unwrap_or(0),
                j.error.as_deref().unwrap_or(""),
            ));
        }
        out
    }
}