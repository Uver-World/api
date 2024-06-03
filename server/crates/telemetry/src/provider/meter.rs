use std::sync::{Arc, Weak};
use opentelemetry::{global, KeyValue, metrics::Result};
use opentelemetry_sdk::metrics::{Aggregation, InstrumentKind, ManualReader, MeterProvider, Pipeline};
use opentelemetry_sdk::metrics::data::{ResourceMetrics, Temporality};
use opentelemetry_sdk::metrics::reader::{AggregationSelector, MetricReader, TemporalitySelector};
use opentelemetry_sdk::Resource;
use crate::worker::MeterWorker;

pub struct SigNozMeter;

impl SigNozMeter {
    pub fn setup(endpoint: String, token: Option<String>) -> MeterWorker {
        // Initialize the provider
        let reader = Self::setup_meter_provider();

        MeterWorker::new(reader, endpoint, token)
    }

    fn setup_meter_provider() -> Arc<ManualReader> {
        let resource = Resource::new(vec![
            KeyValue::new("service.name", "api"),
        ]);

        let reader = Arc::new(ManualReader::builder().build());

        let provider = MeterProvider::builder()
            .with_resource(resource)
            .with_reader(SharedReader(reader.clone()))
            .build();

        global::set_meter_provider(provider);

        reader
    }

}

#[derive(Clone, Debug)]
struct SharedReader(Arc<dyn MetricReader>);

impl TemporalitySelector for SharedReader {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}

impl AggregationSelector for SharedReader {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.0.aggregation(kind)
    }
}

impl MetricReader for SharedReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.0.register_pipeline(pipeline)
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        self.0.collect(rm)
    }

    fn force_flush(&self) -> Result<()> {
        self.0.force_flush()
    }

    fn shutdown(&self) -> Result<()> {
        self.0.shutdown()
    }
}
