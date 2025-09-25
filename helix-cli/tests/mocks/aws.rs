use super::*;
use serde_json::json;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

pub struct MockECR {
    service: MockService,
    region: String,
    account_id: String,
}

impl MockECR {
    pub fn new(region: &str) -> Self {
        let mut mock = Self {
            service: MockService::new(),
            region: region.to_string(),
            account_id: "123456789012".to_string(),
        };
        mock.setup_defaults();
        mock
    }

    pub fn with_account_id(mut self, account_id: &str) -> Self {
        self.account_id = account_id.to_string();
        self
    }

    pub fn mock_create_repository(&self, repo_name: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "repository": {
                    "repositoryArn": format!("arn:aws:ecr:{}:{}:repository/{}",
                        self.region, self.account_id, repo_name),
                    "registryId": self.account_id,
                    "repositoryName": repo_name,
                    "repositoryUri": format!("{}.dkr.ecr.{}.amazonaws.com/{}",
                        self.account_id, self.region, repo_name),
                    "createdAt": "2024-01-01T00:00:00Z",
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateRepository", response);
    }

    pub fn mock_get_authorization_token(&self) {
        let token = BASE64.encode(format!("AWS:mock_password"));
        let response = MockResponse {
            status: 200,
            body: json!({
                "authorizationData": [{
                    "authorizationToken": token,
                    "proxyEndpoint": format!("https://{}.dkr.ecr.{}.amazonaws.com",
                        self.account_id, self.region),
                    "expiresAt": "2024-12-31T23:59:59Z"
                }]
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("GetAuthorizationToken", response);
    }

    pub fn mock_describe_repositories(&self, repositories: Vec<&str>) {
        let repos: Vec<_> = repositories.iter().map(|name| {
            json!({
                "repositoryArn": format!("arn:aws:ecr:{}:{}:repository/{}",
                    self.region, self.account_id, name),
                "registryId": self.account_id,
                "repositoryName": name,
                "repositoryUri": format!("{}.dkr.ecr.{}.amazonaws.com/{}",
                    self.account_id, self.region, name),
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "repositories": repos
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("DescribeRepositories", response);
    }

    pub fn mock_put_image(&self, repo_name: &str, tag: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "image": {
                    "registryId": self.account_id,
                    "repositoryName": repo_name,
                    "imageId": {
                        "imageDigest": format!("sha256:{}", hex::encode(&[0u8; 32])),
                        "imageTag": tag
                    },
                    "imageManifest": "{}",
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("PutImage/{}/{}", repo_name, tag), response);
    }

    pub fn mock_batch_delete_image(&self, repo_name: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "imageIds": [],
                "failures": []
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("BatchDeleteImage/{}", repo_name), response);
    }

    pub fn mock_delete_repository(&self, repo_name: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "repository": {
                    "repositoryArn": format!("arn:aws:ecr:{}:{}:repository/{}",
                        self.region, self.account_id, repo_name),
                    "registryId": self.account_id,
                    "repositoryName": repo_name,
                }
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("DeleteRepository/{}", repo_name), response);
    }
}

impl CloudMock for MockECR {
    fn setup_defaults(&mut self) {
        self.mock_get_authorization_token();
        self.mock_describe_repositories(vec![]);
    }

    fn simulate_failure(&mut self, endpoint: &str, error_code: u16) {
        let response = MockResponse {
            status: error_code,
            body: json!({
                "__type": "RepositoryNotFoundException",
                "message": "The repository does not exist"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(endpoint, response);
    }

    fn verify_auth(&self) -> bool {
        self.service.get_calls().iter()
            .any(|call| call.endpoint == "GetAuthorizationToken")
    }

    fn get_service(&self) -> &MockService {
        &self.service
    }
}

pub struct MockS3 {
    service: MockService,
    region: String,
    bucket_name: String,
}

impl MockS3 {
    pub fn new(region: &str, bucket_name: &str) -> Self {
        let mut mock = Self {
            service: MockService::new(),
            region: region.to_string(),
            bucket_name: bucket_name.to_string(),
        };
        mock.setup_defaults();
        mock
    }

    pub fn mock_create_bucket(&self) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "Location": format!("/{}", self.bucket_name)
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("CreateBucket", response);
    }

    pub fn mock_put_object(&self, key: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "ETag": "\"d41d8cd98f00b204e9800998ecf8427e\""
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("PutObject/{}", key), response);
    }

    pub fn mock_get_object(&self, key: &str, content: &str) {
        let response = MockResponse {
            status: 200,
            body: json!({
                "Body": content,
                "ContentLength": content.len(),
                "ContentType": "application/octet-stream"
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("GetObject/{}", key), response);
    }

    pub fn mock_list_objects(&self, keys: Vec<&str>) {
        let objects: Vec<_> = keys.iter().map(|key| {
            json!({
                "Key": key,
                "Size": 1024,
                "LastModified": "2024-01-01T00:00:00Z"
            })
        }).collect();

        let response = MockResponse {
            status: 200,
            body: json!({
                "Contents": objects
            }),
            headers: HashMap::new(),
        };
        self.service.mock_response("ListObjectsV2", response);
    }

    pub fn mock_delete_object(&self, key: &str) {
        let response = MockResponse {
            status: 204,
            body: json!({}),
            headers: HashMap::new(),
        };
        self.service.mock_response(&format!("DeleteObject/{}", key), response);
    }
}

impl CloudMock for MockS3 {
    fn setup_defaults(&mut self) {
        self.mock_list_objects(vec![]);
    }

    fn simulate_failure(&mut self, endpoint: &str, error_code: u16) {
        let response = MockResponse {
            status: error_code,
            body: json!({
                "Code": "NoSuchBucket",
                "Message": "The specified bucket does not exist"
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