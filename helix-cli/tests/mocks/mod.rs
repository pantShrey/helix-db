use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

pub mod aws;
pub mod fly;
pub mod helix_cloud;
pub mod docker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MockService {
    responses: Arc<Mutex<HashMap<String, MockResponse>>>,
    calls: Arc<Mutex<Vec<MockCall>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockCall {
    pub endpoint: String,
    pub method: String,
    pub body: Option<serde_json::Value>,
    pub timestamp: String,
}

impl MockService {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn mock_response(&self, endpoint: &str, response: MockResponse) {
        self.responses.lock().unwrap().insert(endpoint.to_string(), response);
    }

    pub fn record_call(&self, call: MockCall) {
        self.calls.lock().unwrap().push(call);
    }

    pub fn get_response(&self, endpoint: &str) -> Option<MockResponse> {
        self.responses.lock().unwrap().get(endpoint).cloned()
    }

    pub fn get_calls(&self) -> Vec<MockCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        self.responses.lock().unwrap().clear();
        self.calls.lock().unwrap().clear();
    }
}

pub trait CloudMock {
    fn setup_defaults(&mut self);
    fn simulate_failure(&mut self, endpoint: &str, error_code: u16);
    fn verify_auth(&self) -> bool;
    fn get_service(&self) -> &MockService;
}