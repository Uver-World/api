mod trace_worker;
mod meter_worker;

pub use trace_worker::TraceWorker;
pub use meter_worker::MeterWorker;

pub struct TelemetryWorker {
    span_worker: TraceWorker,
    meter_worker: MeterWorker,
}

impl TelemetryWorker {

    pub fn new(span_worker: TraceWorker, meter_worker: MeterWorker) -> Self {
        Self {
            span_worker,
            meter_worker
        }
    }

    pub fn launch(self) {
        self.span_worker.launch();
        self.meter_worker.launch();
    }

}