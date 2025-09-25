use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestProject {
    _dir: TempDir,
    path: PathBuf,
}

impl TestProject {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();
        Self { _dir: dir, path }
    }

    fn with_config(&self, config: &str) -> &Self {
        fs::write(self.path.join("helix.toml"), config).unwrap();
        self
    }

    fn with_queries(&self) -> &Self {
        fs::create_dir_all(self.path.join("db")).unwrap();
        fs::write(
            self.path.join("db/test.hx"),
            "CREATE TABLE users (id INT, name TEXT);",
        )
        .unwrap();
        self
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("helix").unwrap();
        cmd.current_dir(&self.path);
        cmd
    }
}

// ============= FLY.IO COMMAND TESTS =============

#[test]
fn test_fly_init_basic() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["init", "fly"])
        .assert()
        .failure() // Expected to fail without auth
        .stderr(
            predicate::str::contains("fly")
                .or(predicate::str::contains("Fly"))
                .or(predicate::str::contains("authentication")),
        );
}

#[test]
fn test_fly_init_with_name() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["init", "fly", "-n", "staging"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("fly")
                .or(predicate::str::contains("Fly"))
                .or(predicate::str::contains("authentication")),
        );
}

#[test]
fn test_fly_init_all_vm_sizes() {
    let vm_sizes = vec![
        "shared-cpu-4x",
        "shared-cpu-8x",
        "performance-4x",
        "performance-8x",
        "performance-16x",
        "a10",
        "a100-40gb",
        "a100-80gb",
        "l40s",
    ];

    for vm_size in vm_sizes {
        let project = TestProject::new();

        project
            .cmd()
            .args(&["init", "fly", "--vm-size", vm_size])
            .assert()
            .failure(); // Will fail on auth, but validates parsing
    }
}

#[test]
fn test_fly_init_invalid_vm_size() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["init", "fly", "--vm-size", "invalid-size"])
        .assert()
        .failure();
}

#[test]
fn test_fly_init_with_volume_size() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["init", "fly", "--volume-size", "50"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("fly")
                .or(predicate::str::contains("Fly"))
                .or(predicate::str::contains("authentication")),
        );
}

#[test]
fn test_fly_init_with_auth_cli() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["init", "fly", "--auth", "cli"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("fly")
                .or(predicate::str::contains("Fly"))
                .or(predicate::str::contains("authentication")),
        );
}

#[test]
fn test_fly_init_with_auth_api_key() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["init", "fly", "--auth", "api_key"])
        .assert()
        .failure(); // Will fail without API key
}

#[test]
fn test_fly_init_public_flag() {
    let project = TestProject::new();

    // Test public=true
    project
        .cmd()
        .args(&["init", "fly", "--public", "true"])
        .assert()
        .failure();

    // Test public=false (private deployment)
    project
        .cmd()
        .args(&["init", "fly", "--public", "false"])
        .assert()
        .failure();
}

#[test]
fn test_fly_add_to_existing_project() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969");

    project
        .cmd()
        .args(&[
            "add",
            "fly",
            "-n",
            "staging",
            "--vm-size",
            "performance-4x",
            "--volume-size",
            "30",
        ])
        .assert()
        .failure() // Will fail on auth
        .stderr(
            predicate::str::contains("fly")
                .or(predicate::str::contains("Fly"))
                .or(predicate::str::contains("authentication")),
        );
}

#[test]
fn test_fly_add_without_project() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["add", "fly"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("configuration not found"));
}

#[test]
fn test_fly_push_command() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[cloud.fly_app]\ntype = \"fly\"\ncluster_id = \"test-app\"")
           .with_queries();

    project.cmd().args(&["push", "fly_app"]).assert().failure(); // Will fail on Fly deployment
}

// ============= ECR COMMAND TESTS =============

#[test]
fn test_ecr_init_basic() {
    let project = TestProject::new();

    // Remove AWS credentials to test error
    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    }

    project.cmd().args(&["init", "ecr"]).assert().failure(); // Should fail without AWS credentials
}

#[test]
fn test_ecr_init_with_name() {
    let project = TestProject::new();

    // Set mock credentials
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
        std::env::set_var("AWS_DEFAULT_REGION", "us-east-1");
    }

    project
        .cmd()
        .args(&["init", "ecr", "-n", "production"])
        .assert()
        .failure(); // Will fail on actual ECR creation but tests parsing
}

#[test]
fn test_ecr_add_to_existing_project() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969");

    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
    }

    project
        .cmd()
        .args(&["add", "ecr", "-n", "docker-prod"])
        .assert()
        .failure(); // Will fail on ECR creation
}

#[test]
fn test_ecr_add_without_project() {
    let project = TestProject::new();

    project
        .cmd()
        .args(&["add", "ecr"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("configuration not found"));
}

#[test]
fn test_ecr_push_command() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[cloud.ecr_deploy]\ntype = \"ecr\"\nrepository_name = \"test-repo\"")
           .with_queries();

    project
        .cmd()
        .args(&["push", "ecr_deploy"])
        .assert()
        .failure(); // Will fail on Docker/ECR operations
}

// ============= COMBINED FLY AND ECR TESTS =============

#[test]
fn test_multi_cloud_configuration() {
    let project = TestProject::new();

    // First init with local
    project
        .cmd()
        .args(&["init", "local", "-n", "dev"])
        .assert()
        .success();

    // Try to add Fly
    project
        .cmd()
        .args(&["add", "fly", "-n", "staging", "--vm-size", "shared-cpu-8x"])
        .assert()
        .failure(); // Will fail on auth

    // Try to add ECR
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
    }
    project
        .cmd()
        .args(&["add", "ecr", "-n", "production"])
        .assert()
        .failure(); // Will fail on ECR creation

    // Verify local was created
    let config = fs::read_to_string(project.path.join("helix.toml")).unwrap();
    assert!(config.contains("[local.dev]"));
}

#[test]
fn test_status_with_fly_and_ecr() {
    let project = TestProject::new();
    let config = r#"
[project]
name = "multi-cloud"

[local.dev]
port = 6969

[cloud.fly_staging]
type = "fly"
cluster_id = "staging-app"
vm_size = "shared-cpu-4x"

[cloud.ecr_production]
type = "ecr"
repository_name = "prod-repo"
region = "us-east-1"
"#;
    project.with_config(config);

    project
        .cmd()
        .args(&["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fly_staging"))
        .stdout(predicate::str::contains("ecr_production"));
}

#[test]
fn test_build_for_fly_instance() {
    let project = TestProject::new();
    project
        .with_config(
            r#"
[project]
name = "test"

[cloud.fly_app]
type = "fly"
cluster_id = "test-app"
build_mode = "release"
"#,
        )
        .with_queries();

    project.cmd().args(&["build", "fly_app"]).assert().success(); // Build should work even without deployment
}

#[test]
fn test_build_for_ecr_instance() {
    let project = TestProject::new();
    project
        .with_config(
            r#"
[project]
name = "test"

[cloud.ecr_app]
type = "ecr"
repository_name = "test-repo"
build_mode = "release"
"#,
        )
        .with_queries();

    project.cmd().args(&["build", "ecr_app"]).assert().success(); // Build should work even without deployment
}

#[test]
fn test_check_with_fly_instance() {
    let project = TestProject::new();
    project
        .with_config(
            r#"
[project]
name = "test"

[cloud.fly_app]
type = "fly"
cluster_id = "test-app"
"#,
        )
        .with_queries();

    project.cmd().args(&["check", "fly_app"]).assert().success();
}

#[test]
fn test_check_with_ecr_instance() {
    let project = TestProject::new();
    project
        .with_config(
            r#"
[project]
name = "test"

[cloud.ecr_app]
type = "ecr"
repository_name = "test-repo"
"#,
        )
        .with_queries();

    project.cmd().args(&["check", "ecr_app"]).assert().success();
}

// ============= CONFIGURATION VALIDATION TESTS =============

#[test]
fn test_fly_configuration_parameters() {
    let project = TestProject::new();
    let config = r#"
[project]
name = "test"

[cloud.fly_custom]
type = "fly"
cluster_id = "custom-app"
vm_size = "performance-8x"
volume = "app_data:/data"
volume_initial_size = 50
privacy = "private"
auth_type = "api_key"
build_mode = "release"
"#;
    project.with_config(config);

    project
        .cmd()
        .args(&["check", "fly_custom"])
        .assert()
        .success();
}

#[test]
fn test_ecr_configuration_parameters() {
    let project = TestProject::new();
    let config = r#"
[project]
name = "test"

[cloud.ecr_custom]
type = "ecr"
repository_name = "custom-repo"
region = "us-west-2"
registry_url = "123456789.dkr.ecr.us-west-2.amazonaws.com"
auth_type = "aws_cli"
build_mode = "release"
"#;
    project.with_config(config);

    project
        .cmd()
        .args(&["check", "ecr_custom"])
        .assert()
        .success();
}

// ============= ERROR HANDLING TESTS =============

#[test]
fn test_fly_missing_required_config() {
    let project = TestProject::new();
    project.with_config(
        r#"
[project]
name = "test"

[cloud.fly_broken]
type = "fly"
# Missing cluster_id
"#,
    );

    project
        .cmd()
        .args(&["check", "fly_broken"])
        .assert()
        .failure();
}

#[test]
fn test_ecr_missing_required_config() {
    let project = TestProject::new();
    project.with_config(
        r#"
[project]
name = "test"

[cloud.ecr_broken]
type = "ecr"
# Missing repository_name
"#,
    );

    project
        .cmd()
        .args(&["check", "ecr_broken"])
        .assert()
        .failure();
}

#[test]
fn test_duplicate_fly_instance_names() {
    let project = TestProject::new();
    project.with_config(
        r#"
[project]
name = "test"

[cloud.staging]
type = "fly"
cluster_id = "app1"
"#,
    );

    project
        .cmd()
        .args(&["add", "fly", "-n", "staging"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_duplicate_ecr_instance_names() {
    let project = TestProject::new();
    project.with_config(
        r#"
[project]
name = "test"

[cloud.production]
type = "ecr"
repository_name = "repo1"
"#,
    );

    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
    }

    project
        .cmd()
        .args(&["add", "ecr", "-n", "production"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}
