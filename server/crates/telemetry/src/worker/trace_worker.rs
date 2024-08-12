use std::sync::mpsc::Receiver;
use std::thread;
use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry_sdk::export::trace::SpanData;
use reqwest::Client;
use tokio::runtime::Runtime;
use crate::provider::SigNozExportError;

pub struct TraceWorker {
    receiver: Receiver<Vec<SpanData>>,
    endpoint: String,
    client: Client
}

impl TraceWorker {

    pub fn new(receiver: Receiver<Vec<SpanData>>, endpoint: String) -> Self {
        Self {
            receiver,
            endpoint,
            client: Client::new()
        }
    }

    pub fn launch(self) {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            rt.block_on(async {
                while let Ok(batch) = self.receiver.recv() {
                    self.process_batch(batch).await;
                }
            });
        });
    }

    async fn process_batch(&self, batch: Vec<SpanData>) {
        println!("data {:?}", batch);
        if let Ok(data) = Self::build_body(batch) {
            self.send_request(data).await;
        }
    }

    // Send the serialized data to SigNoz
    async fn send_request(&self, (data, content_type): (Vec<u8>, &'static str)) {
        let endpoint = &self.endpoint;
        let res = self.client.post(format!("{endpoint}/v1/traces"))
            .header("Content-Type", content_type)
            .body(data)
            .send()
            .await;

        match res {
            Ok(response ) => {
                eprintln!("SUCCESS: {:?}", response);
            }
            Err(err) => {
                eprintln!("Failed to send telemetry data to SigNoz: {}", err);
            }
        }
    }

    fn build_body(spans: Vec<SpanData>) -> TraceResult<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
        use prost::Message;

        let req = ExportTraceServiceRequest {
            resource_spans: spans.into_iter().map(Into::into).collect(),
        };
        let mut buf = vec![];
        req.encode(&mut buf).map_err(|err| TraceError::ExportFailed(Box::new(SigNozExportError::new(&err.to_string()))))?;

        Ok((buf, "application/x-protobuf"))
    }
}