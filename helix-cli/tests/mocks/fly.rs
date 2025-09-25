use super::*;
use serde_json::json;

pub struct MockFly {
    service: MockService,
    app_name: String,
    auth_token: Option<String>,
}

impl MockFly {
    pub fn new(app_name: &str) -> Self {
        let mut mock = Self {
            service: MockService::new(),
            app_name: app_name.to_string(),
            auth_token: None,
        };
        mock.setup_defaults();
        mock
    }

    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    pub fn mock_create_app(&self) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "app": {
                    "id": format!("{}-{}", self.app_name, uuid::Uuid::new_v4()),
                    "name": self.app_name,
                    "organization": {
                        "slug": "personal",
                        "name": "Personal"
                    },
                    "status": "pending",
                    "created_at": "2024-01-01T00:00:00Z"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateApp", response);
    }

    pub fn mock_get_app(&self, status: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "app": {
                    "id": format!("{}-{}", self.app_name, uuid::Uuid::new_v4()),
                    "name": self.app_name,
                    "status": status,
                    "hostname": format!("{}.fly.dev", self.app_name),
                    "deployed": status == "deployed",
                    "current_release": if status == "deployed" {
                        json!({
                            "id": "v1",
                            "version": 1,
                            "stable": true
                        })
                    } else {
                        json!(null)
                    }
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetApp", response);
    }

    pub fn mock_create_machine(&self, vm_size: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "id": format!("machine-{}", uuid::Uuid::new_v4()),
                "name": format!("{}-machine", self.app_name),
                "state": "created",
                "region": "ewr",
                "instance_id": format!("instance-{}", uuid::Uuid::new_v4()),
                "private_ip": "10.0.0.1",
                "config": {
                    "guest": {
                        "cpu_kind": if vm_size.contains("performance") { "performance" } else { "shared" },
                        "cpus": match vm_size {
                            "shared-cpu-4x" => 4,
                            "shared-cpu-8x" => 8,
                            "performance-4x" => 4,
                            "performance-8x" => 8,
                            "performance-16x" => 16,
                            _ => 4
                        },
                        "memory_mb": match vm_size {
                            "shared-cpu-4x" => 1024,
                            "shared-cpu-8x" => 2048,
                            "performance-4x" => 8192,
                            "performance-8x" => 16384,
                            "performance-16x" => 32768,
                            "a10" | "a100-40gb" | "a100-80gb" | "l40s" => 32768,
                            _ => 1024
                        },
                        "gpu_kind": match vm_size {
                            "a10" => Some("a10"),
                            "a100-40gb" => Some("a100-40gb"),
                            "a100-80gb" => Some("a100-80gb"),
                            "l40s" => Some("l40s"),
                            _ => None
                        }.map(String::from),
                    }
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateMachine", response);
    }

    pub fn mock_create_volume(&self, size_gb: u32) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "volume": {
                    "id": format!("vol_{}", uuid::Uuid::new_v4()),
                    "name": format!("{}_data", self.app_name),
                    "size_gb": size_gb,
                    "region": "ewr",
                    "zone": "ewr-1",
                    "encrypted": true,
                    "created_at": "2024-01-01T00:00:00Z",
                    "state": "created",
                    "attached_machine_id": null,
                    "attached_allocation_id": null
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateVolume", response);
    }

    pub fn mock_deploy(&self, image_url: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "release": {
                    "id": format!("release-{}", uuid::Uuid::new_v4()),
                    "version": 1,
                    "stable": false,
                    "description": "Deployment from Helix CLI",
                    "deployment_strategy": "rolling",
                    "image_ref": image_url,
                    "created_at": "2024-01-01T00:00:00Z"
                },
                "status": "in_progress"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("Deploy", response);
    }

    pub fn mock_get_deployment_status(&self, status: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "status": status,
                "successful": status == "successful",
                "description": match status {
                    "successful" => "Deployment completed successfully",
                    "failed" => "Deployment failed",
                    _ => "Deployment in progress"
                },
                "version": 1,
                "created_at": "2024-01-01T00:00:00Z"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetDeploymentStatus", response);
    }

    pub fn mock_list_machines(&self, count: usize) {
        let machines: Vec<_> = (0..count).map(|i| {
            json!({
                "id": format!("machine-{}", i),
                "name": format!("{}-machine-{}", self.app_name, i),
                "state": "started",
                "region": "ewr",
                "instance_id": format!("instance-{}", i),
                "private_ip": format!("10.0.0.{}", i + 1),
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "machines": machines,
                "total_count": count
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("ListMachines", response);
    }

    pub fn mock_stop_machine(&self, machine_id: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ok": true,
                "machine_id": machine_id,
                "state": "stopped"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("StopMachine/{}", machine_id), response);
    }

    pub fn mock_start_machine(&self, machine_id: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ok": true,
                "machine_id": machine_id,
                "state": "started"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("StartMachine/{}", machine_id), response);
    }

    pub fn mock_destroy_machine(&self, machine_id: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ok": true,
                "machine_id": machine_id
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("DestroyMachine/{}", machine_id), response);
    }

    pub fn mock_delete_app(&self) {
        let response = MockResponse {
            status: 204,
            body: json!({}),
            headers: HashMap::new(),
        };
        self.service.mock_response("DeleteApp", response);
    }

    pub fn mock_get_logs(&self, log_lines: Vec<&str>) {
        let logs: Vec<_> = log_lines.iter().map(|line| {
            json!({
                "timestamp": "2024-01-01T00:00:00Z",
                "message": line,
                "level": "info",
                "instance": "machine-1",
                "region": "ewr"
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "logs": logs
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetLogs", response);
    }

    pub fn mock_get_secrets(&self, secrets: HashMap<&str, &str>) {
        let secret_list: Vec<_> = secrets.keys().map(|key| {
            json!({
                "name": key,
                "digest": format!("sha256:{}", hex::encode(&[0u8; 8])),
                "created_at": "2024-01-01T00:00:00Z"
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "secrets": secret_list
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetSecrets", response);
    }

    pub fn mock_set_secret(&self, name: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "secret": {
                    "name": name,
                    "digest": format!("sha256:{}", hex::encode(&[0u8; 8])),
                    "created_at": "2024-01-01T00:00:00Z"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("SetSecret/{}", name), response);
    }
}

impl CloudMock for MockFly {
    fn setup_defaults(&mut self) {
        self.mock_get_app("deployed");
        self.mock_list_machines(0);
    }

    fn simulate_failure(&mut self, endpoint: &str, error_code: u16) {
        let response = MockResponse {
            status: error_code,
            body: json!({
                "error": match error_code {
                    404 => "App not found",
                    401 => "Unauthorized",
                    429 => "Rate limited",
                    500 => "Internal server error",
                    _ => "Unknown error"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(endpoint, response);
    }

    fn verify_auth(&self) -> bool {
        self.auth_token.is_some()
    }

    fn get_service(&self) -> &MockService {
        &self.service
    }
}