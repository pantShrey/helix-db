# Helix CLI Cloud Commands Test Plan

## Test Strategy

### 1. Test Categories
- **Unit Tests**: Individual command parsing and validation
- **Integration Tests**: Command execution with mocked cloud services
- **End-to-End Tests**: Full deployment cycles with real cloud services (manual/CI)
- **Error Handling Tests**: Invalid inputs and failure scenarios

### 2. Test Environment Setup
- Mock cloud services for automated tests
- Test project templates and configurations
- Docker environment for containerized tests
- CI/CD pipeline integration

## Detailed Test Cases

### `init` Command Tests

#### Test Suite: init_helix_tests
```rust
// Test cases for helix init helix
1. test_init_helix_default_region()
   - Command: `helix init helix`
   - Verify: Default region (us-east-1), project structure created

2. test_init_helix_custom_region()
   - Command: `helix init helix --region us-west-2`
   - Verify: Custom region set, config file correct

3. test_init_helix_with_name()
   - Command: `helix init helix -n production`
   - Verify: Named instance created, config section correct

4. test_init_helix_with_template()
   - Command: `helix init helix --template python`
   - Verify: Python template created, helix-python directory exists

5. test_init_helix_custom_queries_path()
   - Command: `helix init helix -q ./custom/queries/`
   - Verify: Queries directory at custom location

6. test_init_helix_existing_project()
   - Setup: Existing helix.toml
   - Command: `helix init helix`
   - Verify: Error - project already exists

7. test_init_helix_invalid_region()
   - Command: `helix init helix --region invalid-region`
   - Verify: Error - invalid region format
```

#### Test Suite: init_ecr_tests
```rust
// Test cases for helix init ecr
1. test_init_ecr_default()
   - Command: `helix init ecr`
   - Verify: ECR config created, repository name generated

2. test_init_ecr_with_name()
   - Command: `helix init ecr -n production`
   - Verify: Named ECR instance, repository name formatted correctly

3. test_init_ecr_aws_not_configured()
   - Setup: No AWS credentials
   - Command: `helix init ecr`
   - Verify: Error - AWS not configured

4. test_init_ecr_with_path()
   - Command: `helix init ecr -p ./project/path`
   - Verify: Project created at specified path
```

#### Test Suite: init_fly_tests
```rust
// Test cases for helix init fly
1. test_init_fly_default()
   - Command: `helix init fly`
   - Verify: Default VM size (shared-cpu-4x), volume 20GB, public=true

2. test_init_fly_cli_auth()
   - Command: `helix init fly --auth cli`
   - Verify: CLI authentication mode set

3. test_init_fly_api_key_auth()
   - Command: `helix init fly --auth api_key`
   - Verify: API key auth mode, helix.env created

4. test_init_fly_custom_vm_sizes()
   - Test each VM size:
     * `helix init fly --vm-size shared-cpu-4x`
     * `helix init fly --vm-size shared-cpu-8x`
     * `helix init fly --vm-size performance-4x`
     * `helix init fly --vm-size performance-8x`
     * `helix init fly --vm-size performance-16x`
     * `helix init fly --vm-size a10`
     * `helix init fly --vm-size a100-40gb`
     * `helix init fly --vm-size a100-80gb`
     * `helix init fly --vm-size l40s`
   - Verify: Each VM size correctly configured

5. test_init_fly_invalid_vm_size()
   - Command: `helix init fly --vm-size invalid-size`
   - Verify: Error - invalid VM size

6. test_init_fly_custom_volume_size()
   - Command: `helix init fly --volume-size 50`
   - Verify: Volume size set to 50GB

7. test_init_fly_private_deployment()
   - Command: `helix init fly --public false`
   - Verify: Privacy set to private

8. test_init_fly_with_name()
   - Command: `helix init fly -n staging`
   - Verify: Named Fly instance created
```

#### Test Suite: init_local_tests
```rust
// Test cases for helix init local
1. test_init_local_default()
   - Command: `helix init local`
   - Verify: Local config with default port 6969

2. test_init_local_with_name()
   - Command: `helix init local -n dev`
   - Verify: Named local instance

3. test_init_local_with_template()
   - Command: `helix init local --template python`
   - Verify: Python template with local config
```

### `add` Command Tests

#### Test Suite: add_command_tests
```rust
// Test cases for helix add
1. test_add_helix_to_existing()
   - Setup: Existing project with local instance
   - Command: `helix add helix -n production`
   - Verify: Helix cloud instance added to config

2. test_add_ecr_to_existing()
   - Setup: Existing project
   - Command: `helix add ecr -n docker-deploy`
   - Verify: ECR instance added

3. test_add_fly_to_existing()
   - Setup: Existing project
   - Command: `helix add fly --vm-size performance-8x -n staging`
   - Verify: Fly instance added with correct config

4. test_add_duplicate_name()
   - Setup: Project with instance named "production"
   - Command: `helix add helix -n production`
   - Verify: Error - duplicate instance name

5. test_add_no_existing_project()
   - Setup: No helix.toml
   - Command: `helix add helix`
   - Verify: Error - no existing project

6. test_add_multiple_instances()
   - Setup: Empty project
   - Commands:
     * `helix add local -n dev`
     * `helix add fly -n staging`
     * `helix add helix -n production`
   - Verify: All three instances in config
```

### `push` Command Tests

#### Test Suite: push_command_tests
```rust
// Test cases for helix push
1. test_push_helix_instance()
   - Setup: Helix cloud instance configured
   - Command: `helix push production`
   - Verify: Queries uploaded to cloud

2. test_push_ecr_instance()
   - Setup: ECR instance configured
   - Command: `helix push ecr-deploy`
   - Verify: Docker image built and pushed to ECR

3. test_push_fly_instance()
   - Setup: Fly instance configured
   - Command: `helix push staging`
   - Verify: App deployed to Fly.io

4. test_push_local_instance()
   - Setup: Local instance configured
   - Command: `helix push dev`
   - Verify: Docker container started locally

5. test_push_nonexistent_instance()
   - Command: `helix push nonexistent`
   - Verify: Error - instance not found

6. test_push_no_queries()
   - Setup: Instance with no .hql files
   - Command: `helix push production`
   - Verify: Warning or error about no queries

7. test_push_compile_errors()
   - Setup: Invalid .hql files
   - Command: `helix push production`
   - Verify: Error - compilation failed
```

### `build` Command Tests

#### Test Suite: build_command_tests
```rust
// Test cases for helix build
1. test_build_debug_mode()
   - Setup: Local instance with debug mode
   - Command: `helix build dev`
   - Verify: Debug build artifacts created

2. test_build_release_mode()
   - Setup: Production instance with release mode
   - Command: `helix build production`
   - Verify: Release build artifacts created

3. test_build_docker_generation()
   - Setup: ECR/Fly instance
   - Command: `helix build docker-instance`
   - Verify: Dockerfile and docker-compose.yml created

4. test_build_with_templates()
   - Setup: Project with Python template
   - Command: `helix build production`
   - Verify: Python client code generated

5. test_build_invalid_queries()
   - Setup: Syntax errors in .hql files
   - Command: `helix build production`
   - Verify: Error with detailed compilation messages
```

### `auth` Command Tests

#### Test Suite: auth_command_tests
```rust
// Test cases for helix auth
1. test_auth_login_success()
   - Command: `helix auth login`
   - Mock: GitHub OAuth flow
   - Verify: Credentials saved to ~/.helix/credentials

2. test_auth_logout()
   - Setup: Existing credentials
   - Command: `helix auth logout`
   - Verify: Credentials removed

3. test_auth_create_key()
   - Setup: Logged in user
   - Command: `helix auth create-key --cluster cluster-id`
   - Verify: API key generated and displayed

4. test_auth_not_logged_in()
   - Setup: No credentials
   - Command: `helix auth create-key --cluster id`
   - Verify: Error - not authenticated
```

### `pull` Command Tests

```rust
// Test cases for helix pull
1. test_pull_from_helix_cloud()
   - Setup: Helix cloud instance
   - Command: `helix pull production`
   - Verify: .hql files downloaded to queries directory

2. test_pull_overwrite_local()
   - Setup: Local .hql files exist
   - Command: `helix pull production`
   - Verify: Confirmation prompt, files overwritten

3. test_pull_no_remote_queries()
   - Setup: Empty cloud instance
   - Command: `helix pull production`
   - Verify: Message about no queries to pull
```

### `start/stop` Command Tests

```rust
// Test cases for start/stop
1. test_start_local_instance()
   - Command: `helix start dev`
   - Verify: Docker container running

2. test_stop_local_instance()
   - Setup: Running container
   - Command: `helix stop dev`
   - Verify: Container stopped

3. test_start_already_running()
   - Setup: Container already running
   - Command: `helix start dev`
   - Verify: Message about already running

4. test_stop_not_running()
   - Setup: No running container
   - Command: `helix stop dev`
   - Verify: Message about not running
```

### `delete` Command Tests

```rust
// Test cases for helix delete
1. test_delete_local_instance()
   - Command: `helix delete dev`
   - Verify: Instance removed from config, container deleted

2. test_delete_cloud_instance()
   - Command: `helix delete production`
   - Verify: Confirmation prompt, cloud resources cleaned

3. test_delete_with_confirmation()
   - Command: `helix delete production --yes`
   - Verify: No prompt, immediate deletion

4. test_delete_nonexistent()
   - Command: `helix delete nonexistent`
   - Verify: Error - instance not found
```

### `status` Command Tests

```rust
// Test cases for helix status
1. test_status_all_instances()
   - Setup: Multiple instances configured
   - Command: `helix status`
   - Verify: Table showing all instance statuses

2. test_status_empty_project()
   - Setup: No instances configured
   - Command: `helix status`
   - Verify: Message about no instances

3. test_status_mixed_states()
   - Setup: Some running, some stopped
   - Command: `helix status`
   - Verify: Correct status for each instance
```

### `check` Command Tests

```rust
// Test cases for helix check
1. test_check_all_instances()
   - Command: `helix check`
   - Verify: Validation results for all instances

2. test_check_specific_instance()
   - Command: `helix check production`
   - Verify: Validation for specific instance only

3. test_check_invalid_queries()
   - Setup: Syntax errors in .hql files
   - Command: `helix check`
   - Verify: Detailed error messages

4. test_check_missing_config()
   - Setup: Incomplete configuration
   - Command: `helix check`
   - Verify: Configuration errors reported
```

### `prune` Command Tests

```rust
// Test cases for helix prune
1. test_prune_specific_instance()
   - Command: `helix prune --instance dev`
   - Verify: Only dev instance resources cleaned

2. test_prune_all_instances()
   - Command: `helix prune -a`
   - Verify: All instances cleaned, confirmation required

3. test_prune_docker_cleanup()
   - Setup: Dangling Docker images
   - Command: `helix prune`
   - Verify: Docker resources cleaned

4. test_prune_nothing_to_clean()
   - Setup: Clean state
   - Command: `helix prune`
   - Verify: Message about nothing to prune
```

## Configuration Tests

### Database Configuration Tests
```rust
1. test_vector_config_defaults()
   - Verify: m=16, ef_construction=128, ef_search=768

2. test_vector_config_custom()
   - Config: Custom m, ef_construction, ef_search values
   - Verify: Custom values applied

3. test_db_max_size_config()
   - Config: db_max_size_gb=50
   - Verify: Database size limit enforced

4. test_feature_flags()
   - Config: mcp=false, bm25=false
   - Verify: Features disabled in build
```

## Error Handling Tests

### Invalid Input Tests
```rust
1. test_invalid_command_syntax()
2. test_missing_required_flags()
3. test_conflicting_flags()
4. test_invalid_flag_values()
5. test_malformed_config_file()
```

### Network and Service Failure Tests
```rust
1. test_network_timeout()
2. test_auth_failure()
3. test_cloud_service_unavailable()
4. test_docker_daemon_not_running()
5. test_disk_space_full()
```

## Integration Test Scenarios

### Scenario 1: Complete Multi-Environment Setup
```bash
# Initialize project with local development
helix init local -n dev --template python

# Add staging environment on Fly.io
helix add fly -n staging --vm-size shared-cpu-4x --auth cli

# Add production on Helix Cloud
helix add helix -n production --region us-east-1

# Build and deploy to all environments
helix build dev && helix push dev
helix build staging && helix push staging
helix build production && helix push production

# Verify all instances running
helix status
```

### Scenario 2: ECR Deployment Pipeline
```bash
# Initialize ECR project
helix init ecr -n docker-deploy

# Build and push to ECR
helix build docker-deploy
helix push docker-deploy

# Pull updated queries
helix pull docker-deploy

# Clean up
helix delete docker-deploy --yes
```

### Scenario 3: Template-Based Development
```bash
# Initialize with Python template
helix init local --template python

# Add cloud deployment
helix add helix -n cloud

# Develop locally
helix build local
helix push local

# Deploy to cloud
helix build cloud
helix push cloud
```

## Performance Tests

### Load Tests
```rust
1. test_large_query_compilation()
   - Test with 100+ .hql files

2. test_large_docker_build()
   - Test with large dependencies

3. test_parallel_deployments()
   - Deploy to multiple instances simultaneously
```

## Test Implementation Strategy

### Phase 1: Core Command Tests
- Implement unit tests for command parsing
- Mock cloud services for basic integration tests

### Phase 2: Integration Tests
- Docker-based testing environment
- Mock AWS, Fly.io APIs
- Test full command lifecycles

### Phase 3: End-to-End Tests
- CI/CD pipeline with real cloud accounts
- Automated deployment verification
- Performance benchmarking

### Phase 4: Error Recovery Tests
- Failure injection testing
- Network interruption handling
- Rollback scenarios

## Test Automation

### GitHub Actions Workflow
```yaml
name: Cloud Commands Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - Run unit test suite
      - Coverage report

  integration-tests:
    runs-on: ubuntu-latest
    services:
      docker:
        image: docker:dind
    steps:
      - Run integration tests
      - Mock cloud service tests

  e2e-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
      - Run end-to-end tests
      - Deploy to test environments
      - Verify deployments
      - Clean up resources
```

## Manual Testing Checklist

### Pre-release Testing
- [ ] All command combinations tested
- [ ] All error scenarios handled
- [ ] Documentation matches implementation
- [ ] Performance within acceptable limits
- [ ] Security best practices followed
- [ ] No credentials exposed in logs
- [ ] Rollback procedures verified

### User Acceptance Testing
- [ ] New user onboarding flow
- [ ] Migration from existing deployment
- [ ] Multi-environment workflow
- [ ] Error recovery procedures
- [ ] Documentation clarity

## Success Criteria

1. **Coverage**: >90% code coverage for cloud commands
2. **Reliability**: All tests pass consistently
3. **Performance**: Commands complete within timeout limits
4. **Error Handling**: All error paths tested and documented
5. **Security**: No credential leaks, secure by default
6. **Documentation**: All features documented with examples