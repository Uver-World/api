use crate::provider::exporter_error::SigNozExportError;
use crate::worker::TraceWorker;

use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use futures_core::future::BoxFuture;
use opentelemetry::{global, KeyValue};
use opentelemetry::trace::TraceError;
use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{Config, TracerProvider};


#[derive(Clone, Debug)]
pub struct SigNozTracer {
    sender: Sender<Vec<SpanData>>,
}

impl SigNozTracer {
    fn new(sender: Sender<Vec<SpanData>>) -> Self {
        Self {
            sender
        }
    }

    pub fn setup(endpoint: String) -> TraceWorker {
        // Create the channel
        let (sender, receiver) = mpsc::channel();

        // Initialize the provider
        Self::new(sender).setup_provider();

        // Create a trace worker
        TraceWorker::new(receiver, endpoint)
    }

    fn setup_provider(self) {
        let provider = TracerProvider::builder()
            .with_simple_exporter(self).with_config(
            Config::default().with_resource(
                Resource::new(vec![
                    KeyValue::new("service.name", "api")
                ])
            ))
            .build();

        global::set_tracer_provider(provider);
    }

}

impl SpanExporter for SigNozTracer {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let sender = self.sender.clone();
        Box::pin(async move {
            match sender.send(batch) {
                Ok(()) => Ok(()),
                Err(err) => Err(TraceError::ExportFailed(Box::new(SigNozExportError::new(&err.to_string())))) // TODO Replace with appropriate error handling
            }
        })
    }
}
