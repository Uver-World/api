mod provider;
mod worker;
mod settings;

use crate::provider::{SigNozMeter, SigNozTracer};
use crate::worker::TelemetryWorker;

pub use settings::TelemetrySettings;

pub fn start_telemetry(telemetry_settings: TelemetrySettings)  {
    let trace_worker = SigNozTracer::setup(telemetry_settings.hostname.clone());
    let meter_worker = SigNozMeter::setup(telemetry_settings.hostname.clone(), telemetry_settings.token.clone());

    TelemetryWorker::new(trace_worker, meter_worker).launch();
}
