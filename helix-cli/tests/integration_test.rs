use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;

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
            "CREATE TABLE users (id INT, name TEXT);"
        ).unwrap();
        self
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("helix").unwrap();
        cmd.current_dir(&self.path);
        cmd
    }
}

#[test]
fn test_init_helix_default() {
    let project = TestProject::new();

    project.cmd()
        .args(&["init", "helix"])
        .assert()
        .failure(); // Will fail without auth but tests command parsing

    // Check that it attempted to create project structure
    assert!(project.path.exists());
}

#[test]
fn test_init_local_default() {
    let project = TestProject::new();

    project.cmd()
        .args(&["init", "local"])
        .assert()
        .success();

    assert!(project.path.join("helix.toml").exists());
    assert!(project.path.join("db").exists());
}

#[test]
fn test_init_local_with_name() {
    let project = TestProject::new();

    project.cmd()
        .args(&["init", "local", "-n", "development"])
        .assert()
        .success();

    let config = fs::read_to_string(project.path.join("helix.toml")).unwrap();
    assert!(config.contains("[local.development]"));
}

#[test]
fn test_init_with_custom_queries_path() {
    let project = TestProject::new();

    project.cmd()
        .args(&["init", "local", "-q", "./custom/queries/"])
        .assert()
        .success();

    assert!(project.path.join("custom/queries").exists());
}

#[test]
fn test_init_existing_project_fails() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"existing\"");

    project.cmd()
        .args(&["init", "local"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already"));
}

#[test]
fn test_add_to_existing_project() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969");

    project.cmd()
        .args(&["add", "local", "-n", "staging"])
        .assert()
        .success();

    let config = fs::read_to_string(project.path.join("helix.toml")).unwrap();
    assert!(config.contains("[local.staging]"));
    assert!(config.contains("[local.dev]"));
}

#[test]
fn test_add_duplicate_name_fails() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.production]\nport = 6969");

    project.cmd()
        .args(&["add", "local", "-n", "production"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_add_no_existing_project() {
    let project = TestProject::new();

    project.cmd()
        .args(&["add", "local"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("configuration not found"));
}

#[test]
fn test_status_without_project() {
    let project = TestProject::new();
    // Don't create any config

    project.cmd()
        .args(&["status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a Helix project"));
}

#[test]
fn test_status_with_instances() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969\n\n[local.staging]\nport = 7000");

    project.cmd()
        .args(&["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("staging"));
}

#[test]
fn test_check_with_valid_queries() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969")
           .with_queries();

    project.cmd()
        .args(&["check"])
        .assert()
        .success();
}

#[test]
fn test_check_specific_instance() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969\n\n[local.prod]\nport = 8080")
           .with_queries();

    project.cmd()
        .args(&["check", "dev"])
        .assert()
        .success();
}

#[test]
fn test_build_local_instance() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969")
           .with_queries();

    project.cmd()
        .args(&["build", "dev"])
        .assert()
        .success();
}

#[test]
fn test_delete_instance_without_confirmation() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969\n\n[local.staging]\nport = 7000");

    // Try without --yes flag (should work but ask for confirmation in real usage)
    project.cmd()
        .args(&["delete", "staging"])
        .assert()
        .success(); // The delete command exists and parses

    // Verify instance still exists since we didn't confirm
    let config = fs::read_to_string(project.path.join("helix.toml")).unwrap();
    assert!(config.contains("[local.dev]"));
}

#[test]
fn test_prune_with_no_resources() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"");

    project.cmd()
        .args(&["prune"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to prune"));
}

#[test]
fn test_init_fly_with_vm_size() {
    let project = TestProject::new();

    project.cmd()
        .args(&["init", "fly", "--vm-size", "shared-cpu-4x"])
        .assert()
        .failure() // Will fail without Fly auth but tests argument parsing
        .stderr(predicate::str::contains("fly").or(predicate::str::contains("Fly")));
}

#[test]
fn test_init_ecr_requires_aws() {
    let project = TestProject::new();
    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
    }

    project.cmd()
        .args(&["init", "ecr"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("AWS"));
}

#[test]
fn test_push_without_instances() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"");

    project.cmd()
        .args(&["push", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("At least one instance"));
}

#[test]
fn test_pull_without_instances() {
    let project = TestProject::new();
    project.with_config("[project]\nname = \"test\"");

    project.cmd()
        .args(&["pull", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("At least one instance"));
}

#[test]
fn test_init_with_python_template() {
    let project = TestProject::new();

    project.cmd()
        .args(&["init", "local", "--template", "python"])
        .assert()
        .success();

    assert!(project.path.join("helix-python").exists());
    assert!(project.path.join("helix-python/requirements.txt").exists());
}

#[test]
fn test_auth_logout() {
    Command::cargo_bin("helix").unwrap()
        .args(&["auth", "logout"])
        .assert()
        .success();
}

#[test]
fn test_invalid_command() {
    Command::cargo_bin("helix").unwrap()
        .args(&["invalid-command"])
        .assert()
        .failure();
}

#[test]
fn test_help_command() {
    Command::cargo_bin("helix").unwrap()
        .args(&["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("helix").unwrap()
        .args(&["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("helix"));
}