use super::*;
use serde_json::json;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

pub struct MockHelixCloud {
    service: MockService,
    cluster_id: String,
    api_key: Option<String>,
    github_token: Option<String>,
}

impl MockHelixCloud {
    pub fn new(cluster_id: &str) -> Self {
        let mut mock = Self {
            service: MockService::new(),
            cluster_id: cluster_id.to_string(),
            api_key: None,
            github_token: None,
        };
        mock.setup_defaults();
        mock
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    pub fn with_github_token(mut self, token: &str) -> Self {
        self.github_token = Some(token.to_string());
        self
    }

    pub fn mock_github_oauth_flow(&self) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "device_code": "device_code_123",
                "user_code": "ABC-123",
                "verification_uri": "https://github.com/login/device",
                "expires_in": 900,
                "interval": 5
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GitHubDeviceCode", response);

        let token_response = MockResponse {
            status: 200,
            body: json!({
                "access_token": "gho_mock_token_123",
                "token_type": "bearer",
                "scope": "user:email"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GitHubAccessToken", token_response);
    }

    pub fn mock_create_cluster(&self, region: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "cluster": {
                    "id": self.cluster_id,
                    "name": format!("helix-cluster-{}", self.cluster_id),
                    "region": region,
                    "status": "provisioning",
                    "endpoint": format!("https://{}.helix.cloud", self.cluster_id),
                    "created_at": "2024-01-01T00:00:00Z"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateCluster", response);
    }

    pub fn mock_get_cluster(&self, status: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "cluster": {
                    "id": self.cluster_id,
                    "name": format!("helix-cluster-{}", self.cluster_id),
                    "region": "us-east-1",
                    "status": status,
                    "endpoint": format!("https://{}.helix.cloud", self.cluster_id),
                    "created_at": "2024-01-01T00:00:00Z",
                    "specs": {
                        "cpu": "4 vCPU",
                        "memory": "16 GB",
                        "storage": "100 GB SSD",
                        "max_vector_dimensions": 4096,
                        "max_documents": 10000000
                    }
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetCluster", response);
    }

    pub fn mock_upload_queries(&self, query_files: Vec<&str>) {
        let files: Vec<_> = query_files.iter().map(|file| {
            json!({
                "name": file,
                "size": 1024,
                "checksum": format!("sha256:{}", hex::encode(&[0u8; 32])),
                "uploaded_at": "2024-01-01T00:00:00Z"
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "uploaded_files": files,
                "compilation_status": "success",
                "compilation_errors": []
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("UploadQueries", response);
    }

    pub fn mock_download_queries(&self, query_files: Vec<(&str, &str)>) {
        let files: Vec<_> = query_files.iter().map(|(name, content)| {
            json!({
                "name": name,
                "content": BASE64.encode(content),
                "size": content.len(),
                "last_modified": "2024-01-01T00:00:00Z"
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "files": files
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("DownloadQueries", response);
    }

    pub fn mock_compile_queries(&self, success: bool) {
        let response = MockResponse {
            status: if success { 200 } else { 400 },
            body: if success {
                json!({
                    "status": "success",
                    "compiled_at": "2024-01-01T00:00:00Z",
                    "warnings": []
                })
            } else {
                json!({
                    "status": "failed",
                    "errors": [
                        {
                            "file": "test.hql",
                            "line": 5,
                            "column": 10,
                            "message": "Syntax error: unexpected token"
                        }
                    ]
                })
            },
            headers: HashMap::new(),
        };
        self.service.mock_response("CompileQueries", response);
    }

    pub fn mock_get_metrics(&self) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "metrics": {
                    "queries_per_second": 150.5,
                    "average_latency_ms": 12.3,
                    "total_documents": 50000,
                    "storage_used_gb": 2.5,
                    "uptime_percentage": 99.99,
                    "active_connections": 5
                },
                "timestamp": "2024-01-01T00:00:00Z"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetMetrics", response);
    }

    pub fn mock_create_api_key(&self, key_name: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "api_key": {
                    "id": format!("key_{}", uuid::Uuid::new_v4()),
                    "name": key_name,
                    "key": format!("hlx_{}", hex::encode(&[0u8; 32])),
                    "created_at": "2024-01-01T00:00:00Z",
                    "last_used": null,
                    "permissions": ["read", "write", "admin"]
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateApiKey", response);
    }

    pub fn mock_list_api_keys(&self, keys: Vec<&str>) {
        let key_list: Vec<_> = keys.iter().map(|name| {
            json!({
                "id": format!("key_{}", uuid::Uuid::new_v4()),
                "name": name,
                "created_at": "2024-01-01T00:00:00Z",
                "last_used": "2024-01-02T00:00:00Z",
                "permissions": ["read", "write"]
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "api_keys": key_list
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("ListApiKeys", response);
    }

    pub fn mock_delete_cluster(&self) {
        let response = MockResponse {
            status: 204,
            body: json!({}),
            headers: HashMap::new(),
        };
        self.service.mock_response("DeleteCluster", response);
    }

    pub fn mock_get_cluster_logs(&self, log_lines: Vec<&str>) {
        let logs: Vec<_> = log_lines.iter().map(|line| {
            json!({
                "timestamp": "2024-01-01T00:00:00Z",
                "level": "info",
                "message": line,
                "service": "helix-db"
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "logs": logs,
                "next_token": null
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetClusterLogs", response);
    }

    pub fn mock_scale_cluster(&self, new_size: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "cluster": {
                    "id": self.cluster_id,
                    "scaling_status": "in_progress",
                    "target_size": new_size,
                    "estimated_completion": "2024-01-01T00:10:00Z"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("ScaleCluster", response);
    }

    pub fn mock_backup_cluster(&self) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "backup": {
                    "id": format!("backup_{}", uuid::Uuid::new_v4()),
                    "cluster_id": self.cluster_id,
                    "status": "in_progress",
                    "size_gb": 2.5,
                    "created_at": "2024-01-01T00:00:00Z"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("BackupCluster", response);
    }

    pub fn mock_restore_backup(&self, backup_id: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "restore": {
                    "backup_id": backup_id,
                    "cluster_id": self.cluster_id,
                    "status": "in_progress",
                    "estimated_completion": "2024-01-01T00:15:00Z"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("RestoreBackup/{}", backup_id), response);
    }
}

impl CloudMock for MockHelixCloud {
    fn setup_defaults(&mut self) {
        self.mock_get_cluster("active");
        self.mock_get_metrics();
    }

    fn simulate_failure(&mut self, endpoint: &str, error_code: u16) {
        let response = MockResponse {
            status: error_code,
            body: json!({
                "error": {
                    "code": match error_code {
                        401 => "UNAUTHORIZED",
                        403 => "FORBIDDEN",
                        404 => "NOT_FOUND",
                        429 => "RATE_LIMITED",
                        500 => "INTERNAL_ERROR",
                        503 => "SERVICE_UNAVAILABLE",
                        _ => "UNKNOWN_ERROR"
                    },
                    "message": match error_code {
                        401 => "Invalid or missing authentication",
                        403 => "Access denied to this resource",
                        404 => "Cluster not found",
                        429 => "Too many requests, please try again later",
                        500 => "Internal server error",
                        503 => "Service temporarily unavailable",
                        _ => "An unknown error occurred"
                    }
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(endpoint, response);
    }

    fn verify_auth(&self) -> bool {
        self.api_key.is_some() || self.github_token.is_some()
    }

    fn get_service(&self) -> &MockService {
        &self.service
    }
}