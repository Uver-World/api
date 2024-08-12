use std::sync::Arc;
use std::thread;
use std::time::Duration;
use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::ManualReader;
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::Resource;
use reqwest::{Client, ClientBuilder, header::HeaderMap, header::HeaderValue};
use tokio::runtime::Runtime;
use crate::provider::SigNozExportError;

pub struct MeterWorker {
    reader: Arc<ManualReader>,
    endpoint: String,
    client: Client,
}

impl MeterWorker {

    pub fn new(reader: Arc<ManualReader>, endpoint: String, token: Option<String>) -> Self {
        
        let mut client = ClientBuilder::new();
        
        if let Some(token) = token {
            let mut headers = HeaderMap::new();
            headers.insert("signoz-access-token", HeaderValue::from_str(&token).unwrap());
            client = client.default_headers(headers);
        }
        
        let client = client.build().unwrap();
        
        Self {
            reader,
            endpoint,
            client
        }
    }

    pub fn launch(self) {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            rt.block_on(async {
                let mut metrics = ResourceMetrics {
                    resource: Resource::default(),
                    scope_metrics: Vec::new()
                };
                while let Ok(_) = self.reader.collect(&mut metrics) {
                    self.process(&mut metrics).await;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        });
    }

    async fn process(&self, metrics: &mut ResourceMetrics) {
        println!("metrics {:?}", metrics);
        if let Ok(data) = Self::build_body(metrics) {
            self.send_request(data).await;
        }
    }

    // Send the serialized data to SigNoz
    async fn send_request(&self, (data, content_type): (Vec<u8>, &'static str)) {
        let endpoint = &self.endpoint;
        
        let res = self.client.post(format!("{endpoint}/v1/metrics"))
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

    fn build_body(metrics: &mut ResourceMetrics) -> TraceResult<(Vec<u8>, &'static str)> {
        use prost::Message;

        let req: opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest =
            (&*metrics).into();
        let mut buf = vec![];
        req.encode(&mut buf).map_err(|err| TraceError::ExportFailed(Box::new(SigNozExportError::new(&err.to_string()))))?;

        Ok((buf, "application/x-protobuf"))
    }
}