use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

mod mocks;

mod test_utils {
    use super::*;
    use std::collections::HashMap;

    pub struct TestProject {
        pub dir: TempDir,
        pub path: PathBuf,
    }

    impl TestProject {
        pub fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let path = dir.path().to_path_buf();
            Self { dir, path }
        }

        pub fn with_config(&self, config: &str) -> &Self {
            fs::write(self.path.join("helix.toml"), config).unwrap();
            self
        }

        pub fn with_queries(&self) -> &Self {
            fs::create_dir_all(self.path.join("db")).unwrap();
            fs::write(
                self.path.join("db/test.hql"),
                "CREATE TABLE users (id INT, name TEXT);"
            ).unwrap();
            self
        }

        pub fn with_env(&self, vars: HashMap<String, String>) -> &Self {
            let env_content = vars.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(self.path.join("helix.env"), env_content).unwrap();
            self
        }

        pub fn cmd(&self) -> Command {
            let mut cmd = Command::cargo_bin("helix").unwrap();
            cmd.current_dir(&self.path);
            cmd
        }
    }

    pub fn mock_aws_credentials() {
        unsafe {
            std::env::set_var("AWS_ACCESS_KEY_ID", "mock_key");
            std::env::set_var("AWS_SECRET_ACCESS_KEY", "mock_secret");
            std::env::set_var("AWS_DEFAULT_REGION", "us-east-1");
        }
    }

    pub fn mock_fly_token() {
        unsafe {
            std::env::set_var("FLY_API_TOKEN", "mock_fly_token");
        }
    }

    pub fn assert_config_contains(project: &TestProject, key: &str, value: &str) {
        let config = fs::read_to_string(project.path.join("helix.toml")).unwrap();
        assert!(config.contains(key), "Config missing key: {}", key);
        assert!(config.contains(value), "Config missing value: {}", value);
    }
}

mod init_tests {
    use super::*;
    use test_utils::*;

    mod helix_cloud {
        use super::*;

        #[test]
        fn test_init_helix_default_region() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "helix"])
                .assert()
                .success();

            assert!(project.path.join("helix.toml").exists());
            assert!(project.path.join("db").exists());
            assert_config_contains(&project, "type = \"helix\"", "region = \"us-east-1\"");
        }

        #[test]
        fn test_init_helix_custom_region() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "helix", "--region", "us-west-2"])
                .assert()
                .success();

            assert_config_contains(&project, "region = \"us-west-2\"", "type = \"helix\"");
        }

        #[test]
        fn test_init_helix_with_name() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "helix", "-n", "production"])
                .assert()
                .success();

            assert_config_contains(&project, "[cloud.production]", "type = \"helix\"");
        }

        #[test]
        fn test_init_helix_with_template() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "helix", "--template", "python"])
                .assert()
                .success();

            assert!(project.path.join("helix-python").exists());
            assert!(project.path.join("helix-python/requirements.txt").exists());
        }

        #[test]
        fn test_init_helix_custom_queries_path() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "helix", "-q", "./custom/queries/"])
                .assert()
                .success();

            assert!(project.path.join("custom/queries").exists());
            assert_config_contains(&project, "queries_path", "./custom/queries/");
        }

        #[test]
        fn test_init_helix_existing_project() {
            let project = TestProject::new();
            project.with_config("[project]\nname = \"existing\"");

            project.cmd()
                .args(&["init", "helix"])
                .assert()
                .failure()
                .stderr(predicate::str::contains("already exists"));
        }

        #[test]
        fn test_init_helix_invalid_region() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "helix", "--region", "invalid-region-format"])
                .assert()
                .failure()
                .stderr(predicate::str::contains("invalid region"));
        }
    }

    mod ecr {
        use super::*;

        #[test]
        fn test_init_ecr_default() {
            let project = TestProject::new();
            mock_aws_credentials();

            project.cmd()
                .args(&["init", "ecr"])
                .assert()
                .success();

            assert_config_contains(&project, "type = \"ecr\"", "repository_name");
        }

        #[test]
        fn test_init_ecr_with_name() {
            let project = TestProject::new();
            mock_aws_credentials();

            project.cmd()
                .args(&["init", "ecr", "-n", "production"])
                .assert()
                .success();

            assert_config_contains(&project, "[cloud.production]", "type = \"ecr\"");
        }

        #[test]
        fn test_init_ecr_aws_not_configured() {
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
        fn test_init_ecr_with_path() {
            let project = TestProject::new();
            mock_aws_credentials();
            let sub_path = project.path.join("subproject");

            Command::cargo_bin("helix").unwrap()
                .args(&["init", "ecr", "-p", sub_path.to_str().unwrap()])
                .assert()
                .success();

            assert!(sub_path.join("helix.toml").exists());
        }
    }

    mod fly {
        use super::*;

        #[test]
        fn test_init_fly_default() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly"])
                .assert()
                .success();

            assert_config_contains(&project, "type = \"fly\"", "vm_size = \"shared-cpu-4x\"");
            assert_config_contains(&project, "volume_initial_size = 20", "privacy = \"public\"");
        }

        #[test]
        fn test_init_fly_cli_auth() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly", "--auth", "cli"])
                .assert()
                .success();

            assert_config_contains(&project, "auth_type = \"cli\"", "type = \"fly\"");
        }

        #[test]
        fn test_init_fly_api_key_auth() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly", "--auth", "api_key"])
                .assert()
                .success();

            assert_config_contains(&project, "auth_type = \"api_key\"", "type = \"fly\"");
            assert!(project.path.join("helix.env").exists());
        }

        #[test]
        fn test_init_fly_vm_sizes() {
            let vm_sizes = vec![
                "shared-cpu-4x", "shared-cpu-8x", "performance-4x",
                "performance-8x", "performance-16x", "a10",
                "a100-40gb", "a100-80gb", "l40s"
            ];

            for vm_size in vm_sizes {
                let project = TestProject::new();

                project.cmd()
                    .args(&["init", "fly", "--vm-size", vm_size])
                    .assert()
                    .success();

                assert_config_contains(&project, "vm_size", vm_size);
            }
        }

        #[test]
        fn test_init_fly_invalid_vm_size() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly", "--vm-size", "invalid-size"])
                .assert()
                .failure()
                .stderr(predicate::str::contains("invalid VM size"));
        }

        #[test]
        fn test_init_fly_custom_volume_size() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly", "--volume-size", "50"])
                .assert()
                .success();

            assert_config_contains(&project, "volume_initial_size = 50", "type = \"fly\"");
        }

        #[test]
        fn test_init_fly_private_deployment() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly", "--public", "false"])
                .assert()
                .success();

            assert_config_contains(&project, "privacy = \"private\"", "type = \"fly\"");
        }

        #[test]
        fn test_init_fly_with_name() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "fly", "-n", "staging"])
                .assert()
                .success();

            assert_config_contains(&project, "[cloud.staging]", "type = \"fly\"");
        }
    }

    mod local {
        use super::*;

        #[test]
        fn test_init_local_default() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "local"])
                .assert()
                .success();

            assert_config_contains(&project, "[local.dev]", "port = 6969");
        }

        #[test]
        fn test_init_local_with_name() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "local", "-n", "development"])
                .assert()
                .success();

            assert_config_contains(&project, "[local.development]", "port = 6969");
        }

        #[test]
        fn test_init_local_with_template() {
            let project = TestProject::new();

            project.cmd()
                .args(&["init", "local", "--template", "python"])
                .assert()
                .success();

            assert!(project.path.join("helix-python").exists());
            assert_config_contains(&project, "[local", "port = 6969");
        }
    }
}

mod add_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_add_helix_to_existing() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969");

        project.cmd()
            .args(&["add", "helix", "-n", "production"])
            .assert()
            .success();

        assert_config_contains(&project, "[cloud.production]", "type = \"helix\"");
        assert_config_contains(&project, "[local.dev]", "port = 6969");
    }

    #[test]
    fn test_add_ecr_to_existing() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"");
        mock_aws_credentials();

        project.cmd()
            .args(&["add", "ecr", "-n", "docker-deploy"])
            .assert()
            .success();

        assert_config_contains(&project, "[cloud.docker-deploy]", "type = \"ecr\"");
    }

    #[test]
    fn test_add_fly_to_existing() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"");

        project.cmd()
            .args(&["add", "fly", "--vm-size", "performance-8x", "-n", "staging"])
            .assert()
            .success();

        assert_config_contains(&project, "[cloud.staging]", "type = \"fly\"");
        assert_config_contains(&project, "vm_size = \"performance-8x\"", "");
    }

    #[test]
    fn test_add_duplicate_name() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"\n\n[cloud.production]\ntype = \"helix\"");

        project.cmd()
            .args(&["add", "helix", "-n", "production"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("duplicate"));
    }

    #[test]
    fn test_add_no_existing_project() {
        let project = TestProject::new();

        project.cmd()
            .args(&["add", "helix"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("no existing project"));
    }

    #[test]
    fn test_add_multiple_instances() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"");
        mock_aws_credentials();

        project.cmd()
            .args(&["add", "local", "-n", "dev"])
            .assert()
            .success();

        project.cmd()
            .args(&["add", "fly", "-n", "staging"])
            .assert()
            .success();

        project.cmd()
            .args(&["add", "helix", "-n", "production"])
            .assert()
            .success();

        assert_config_contains(&project, "[local.dev]", "");
        assert_config_contains(&project, "[cloud.staging]", "type = \"fly\"");
        assert_config_contains(&project, "[cloud.production]", "type = \"helix\"");
    }
}

mod push_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_push_nonexistent_instance() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"");

        project.cmd()
            .args(&["push", "nonexistent"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));
    }

    #[test]
    fn test_push_no_queries() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"\n\n[cloud.production]\ntype = \"helix\"");

        project.cmd()
            .args(&["push", "production"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("no queries"));
    }
}

mod build_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_build_invalid_queries() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969");

        fs::create_dir_all(project.path.join("db")).unwrap();
        fs::write(
            project.path.join("db/invalid.hql"),
            "INVALID SQL SYNTAX HERE"
        ).unwrap();

        project.cmd()
            .args(&["build", "dev"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("compilation"));
    }
}

mod auth_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_auth_logout() {
        let home = dirs::home_dir().unwrap();
        let creds_dir = home.join(".helix");
        fs::create_dir_all(&creds_dir).unwrap();
        fs::write(creds_dir.join("credentials"), "test_token").unwrap();

        Command::cargo_bin("helix").unwrap()
            .args(&["auth", "logout"])
            .assert()
            .success();

        assert!(!creds_dir.join("credentials").exists());
    }

    #[test]
    fn test_auth_not_logged_in() {
        Command::cargo_bin("helix").unwrap()
            .args(&["auth", "create-key", "--cluster", "test-id"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not authenticated"));
    }
}

mod status_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_status_empty_project() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"");

        project.cmd()
            .args(&["status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("no instances"));
    }
}

mod check_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_check_all_instances() {
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
        project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969\n\n[cloud.prod]\ntype = \"helix\"")
               .with_queries();

        project.cmd()
            .args(&["check", "dev"])
            .assert()
            .success();
    }

    #[test]
    fn test_check_invalid_queries() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"\n\n[local.dev]\nport = 6969");

        fs::create_dir_all(project.path.join("db")).unwrap();
        fs::write(
            project.path.join("db/invalid.hql"),
            "INVALID SYNTAX"
        ).unwrap();

        project.cmd()
            .args(&["check"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("error"));
    }
}

mod prune_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_prune_nothing_to_clean() {
        let project = TestProject::new();
        project.with_config("[project]\nname = \"test\"");

        project.cmd()
            .args(&["prune"])
            .assert()
            .success()
            .stdout(predicate::str::contains("nothing to prune"));
    }
}

mod config_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_vector_config_defaults() {
        let project = TestProject::new();

        project.cmd()
            .args(&["init", "local"])
            .assert()
            .success();

        let config = fs::read_to_string(project.path.join("helix.toml")).unwrap();
        assert!(config.contains("m = 16") || !config.contains("m ="));
        assert!(config.contains("ef_construction = 128") || !config.contains("ef_construction"));
    }

    #[test]
    fn test_feature_flags() {
        let project = TestProject::new();
        let config = r#"
[project]
name = "test"
mcp = false
bm25 = false

[local.dev]
port = 6969
"#;
        project.with_config(config);

        project.cmd()
            .args(&["check"])
            .assert()
            .success();
    }
}

mod integration_scenarios {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_complete_multi_environment_setup() {
        let project = TestProject::new();
        mock_aws_credentials();
        mock_fly_token();

        project.cmd()
            .args(&["init", "local", "-n", "dev", "--template", "python"])
            .assert()
            .success();

        project.cmd()
            .args(&["add", "fly", "-n", "staging", "--vm-size", "shared-cpu-4x", "--auth", "cli"])
            .assert()
            .success();

        project.cmd()
            .args(&["add", "helix", "-n", "production", "--region", "us-east-1"])
            .assert()
            .success();

        project.with_queries();

        project.cmd()
            .args(&["check"])
            .assert()
            .success();

        project.cmd()
            .args(&["status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("dev"))
            .stdout(predicate::str::contains("staging"))
            .stdout(predicate::str::contains("production"));
    }

    #[test]
    fn test_template_based_development() {
        let project = TestProject::new();

        project.cmd()
            .args(&["init", "local", "--template", "python"])
            .assert()
            .success();

        assert!(project.path.join("helix-python").exists());
        assert!(project.path.join("helix-python/requirements.txt").exists());

        project.cmd()
            .args(&["add", "helix", "-n", "cloud"])
            .assert()
            .success();

        project.with_queries();

        project.cmd()
            .args(&["build", "dev"])
            .assert()
            .success();
    }
}

mod error_handling_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_invalid_command_syntax() {
        Command::cargo_bin("helix").unwrap()
            .args(&["invalid", "command"])
            .assert()
            .failure();
    }

    #[test]
    fn test_missing_required_flags() {
        Command::cargo_bin("helix").unwrap()
            .args(&["auth", "create-key"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("cluster"));
    }

    #[test]
    fn test_malformed_config_file() {
        let project = TestProject::new();
        fs::write(project.path.join("helix.toml"), "invalid toml {{}").unwrap();

        project.cmd()
            .args(&["status"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("parse"));
    }
}