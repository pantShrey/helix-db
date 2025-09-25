use super::*;
use serde_json::json;

pub struct MockDocker {
    service: MockService,
}

impl MockDocker {
    pub fn new() -> Self {
        let mut mock = Self {
            service: MockService::new(),
        };
        mock.setup_defaults();
        mock
    }

    pub fn mock_build_image(&self, tag: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "stream": [
                    {"stream": "Step 1/10 : FROM rust:1.70\n"},
                    {"stream": "Step 2/10 : WORKDIR /app\n"},
                    {"stream": "Step 3/10 : COPY . .\n"},
                    {"stream": "Step 10/10 : CMD [\"helix\"]\n"},
                    {"stream": format!("Successfully built {}\n", "abc123def456")},
                    {"stream": format!("Successfully tagged {}\n", tag)}
                ]
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("BuildImage", response);
    }

    pub fn mock_push_image(&self, tag: &str, registry: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "status": "success",
                "digest": format!("sha256:{}", hex::encode(&[0u8; 32])),
                "repository": format!("{}/{}", registry, tag),
                "tag": "latest"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("PushImage/{}", tag), response);
    }

    pub fn mock_create_container(&self, name: &str, image: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "Id": format!("container_{}", hex::encode(&[0u8; 16])),
                "Name": name,
                "Image": image,
                "Created": "2024-01-01T00:00:00Z",
                "Status": "created"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("CreateContainer/{}", name), response);
    }

    pub fn mock_start_container(&self, container_id: &str) {
        let response = MockResponse {
            status: 204,
            body: json!({}),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("StartContainer/{}", container_id), response);
    }

    pub fn mock_stop_container(&self, container_id: &str) {
        let response = MockResponse {
            status: 204,
            body: json!({}),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("StopContainer/{}", container_id), response);
    }

    pub fn mock_remove_container(&self, container_id: &str) {
        let response = MockResponse {
            status: 204,
            body: json!({}),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("RemoveContainer/{}", container_id), response);
    }

    pub fn mock_list_containers(&self, containers: Vec<(&str, &str, &str)>) {
        let container_list: Vec<_> = containers.iter().map(|(id, name, status)| {
            json!({
                "Id": id,
                "Names": [format!("/{}", name)],
                "Image": "helix:latest",
                "State": status,
                "Status": match status.as_ref() {
                    "running" => "Up 5 minutes",
                    "exited" => "Exited (0) 10 minutes ago",
                    _ => "Created"
                },
                "Ports": if *status == "running" {
                    json!([{"PrivatePort": 6969, "PublicPort": 6969, "Type": "tcp"}])
                } else {
                    json!([])
                }
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!(container_list),
            headers: HashMap::new(),
        };
        self.service.mock_response("ListContainers", response);
    }

    pub fn mock_inspect_container(&self, container_id: &str, running: bool) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "Id": container_id,
                "Created": "2024-01-01T00:00:00Z",
                "State": {
                    "Status": if running { "running" } else { "exited" },
                    "Running": running,
                    "StartedAt": "2024-01-01T00:00:00Z",
                    "FinishedAt": if running { "" } else { "2024-01-01T00:05:00Z" }
                },
                "Config": {
                    "Image": "helix:latest",
                    "Env": [
                        "HELIX_PORT=6969",
                        "HELIX_MODE=production"
                    ],
                    "ExposedPorts": {
                        "6969/tcp": {}
                    }
                },
                "NetworkSettings": {
                    "Ports": {
                        "6969/tcp": [{"HostPort": "6969"}]
                    }
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("InspectContainer/{}", container_id), response);
    }

    pub fn mock_container_logs(&self, container_id: &str, log_lines: Vec<&str>) {
        let logs = log_lines.join("\n");
        let response = MockResponse {
            status: 200,
            body: json!({
                "logs": logs,
                "timestamps": true
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("ContainerLogs/{}", container_id), response);
    }

    pub fn mock_list_images(&self, images: Vec<(&str, u64)>) {
        let image_list: Vec<_> = images.iter().map(|(tag, size)| {
            json!({
                "Id": format!("sha256:{}", hex::encode(&[0u8; 32])),
                "RepoTags": [tag],
                "Created": "2024-01-01T00:00:00Z",
                "Size": size,
                "VirtualSize": size
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!(image_list),
            headers: HashMap::new(),
        };
        self.service.mock_response("ListImages", response);
    }

    pub fn mock_remove_image(&self, image_id: &str) {
        let response = MockResponse {
            status: 200,
            body: json!([
                {"Untagged": format!("{}:latest", image_id)},
                {"Deleted": format!("sha256:{}", hex::encode(&[0u8; 32]))}
            ]),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("RemoveImage/{}", image_id), response);
    }

    pub fn mock_prune_containers(&self) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ContainersDeleted": ["container_1", "container_2"],
                "SpaceReclaimed": 1073741824
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("PruneContainers", response);
    }

    pub fn mock_prune_images(&self) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ImagesDeleted": [
                    {"Deleted": "sha256:abc123"},
                    {"Deleted": "sha256:def456"}
                ],
                "SpaceReclaimed": "2147483648"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("PruneImages", response);
    }

    pub fn mock_docker_info(&self) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ID": "ABCD:EFGH:IJKL",
                "Containers": 5,
                "ContainersRunning": 2,
                "Images": 10,
                "ServerVersion": "24.0.0",
                "OperatingSystem": "Docker Desktop",
                "Architecture": "x86_64",
                "MemTotal": "8589934592",
                "DockerRootDir": "/var/lib/docker"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("DockerInfo", response);
    }

    pub fn mock_create_network(&self, name: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "Id": format!("network_{}", hex::encode(&[0u8; 16])),
                "Name": name,
                "Driver": "bridge",
                "Scope": "local"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("CreateNetwork/{}", name), response);
    }

    pub fn mock_create_volume(&self, name: &str) {
        let response = MockResponse {
            status: 201,
            body: json!({
                "Name": name,
                "Driver": "local",
                "Mountpoint": format!("/var/lib/docker/volumes/{}", name),
                "CreatedAt": "2024-01-01T00:00:00Z"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("CreateVolume/{}", name), response);
    }
}

impl CloudMock for MockDocker {
    fn setup_defaults(&mut self) {
        self.mock_docker_info();
        self.mock_list_containers(vec![]);
        self.mock_list_images(vec![]);
    }

    fn simulate_failure(&mut self, endpoint: &str, error_code: u16) {
        let response = MockResponse {
            status: error_code,
            body: json!({
                "message": match error_code {
                    404 => "No such container or image",
                    409 => "Conflict: container or image already exists",
                    500 => "Docker daemon error",
                    503 => "Docker daemon is not running",
                    _ => "Unknown error"
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(endpoint, response);
    }

    fn verify_auth(&self) -> bool {
        true
    }

    fn get_service(&self) -> &MockService {
        &self.service
    }
}