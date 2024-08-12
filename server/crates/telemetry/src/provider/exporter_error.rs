use opentelemetry_sdk::export::ExportError;
use std::fmt;

#[derive(Debug)]
pub struct SigNozExportError {
    message: String,
}

impl SigNozExportError {
    pub fn new(message: &str) -> Self {
        SigNozExportError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for SigNozExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SigNoz Export Error: {}", self.message)
    }
}

impl std::error::Error for SigNozExportError {}

impl ExportError for SigNozExportError {
    fn exporter_name(&self) -> &'static str {
        "SigNozExportError"
    }
}
