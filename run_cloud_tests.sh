#!/bin/bash

# Helix CLI Cloud Commands Test Runner
# This script runs comprehensive tests for all cloud-specific commands

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_FILTER="${1:-}"
VERBOSE="${VERBOSE:-false}"
PARALLEL="${PARALLEL:-true}"
COVERAGE="${COVERAGE:-false}"

print_header() {
    echo -e "\n${BLUE}=================================================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=================================================================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to run tests with proper environment setup
run_test_suite() {
    local suite_name=$1
    local test_pattern=$2

    print_header "Running $suite_name Tests"

    # Set up test environment variables
    export HELIX_TEST_MODE=1
    export MOCK_CLOUD_SERVICES=1
    export TEST_TEMP_DIR=$(mktemp -d)

    # Clean up function
    cleanup() {
        rm -rf "$TEST_TEMP_DIR"
    }
    trap cleanup EXIT

    # Build flags
    local test_args="--test cloud_commands_test"

    if [ -n "$test_pattern" ]; then
        test_args="$test_args $test_pattern"
    fi

    if [ "$VERBOSE" = "true" ]; then
        test_args="$test_args --nocapture"
    fi

    if [ "$PARALLEL" = "false" ]; then
        test_args="$test_args --test-threads=1"
    fi

    # Run tests
    if [ "$COVERAGE" = "true" ]; then
        print_warning "Running with code coverage..."
        cargo tarpaulin --out Html --output-dir target/coverage $test_args
    else
        cargo test $test_args
    fi

    print_success "$suite_name tests completed"
}

# Function to run unit tests
run_unit_tests() {
    print_header "Unit Tests"

    local suites=(
        "init_tests::helix_cloud"
        "init_tests::ecr"
        "init_tests::fly"
        "init_tests::local"
        "add_tests"
        "push_tests"
        "build_tests"
        "auth_tests"
        "status_tests"
        "check_tests"
        "prune_tests"
        "config_tests"
    )

    for suite in "${suites[@]}"; do
        if [ -z "$TEST_FILTER" ] || [[ "$suite" == *"$TEST_FILTER"* ]]; then
            echo -e "\n${YELLOW}Running $suite...${NC}"
            cargo test --test cloud_commands_test "$suite" -- --quiet || {
                print_error "Failed: $suite"
                exit 1
            }
            print_success "Passed: $suite"
        fi
    done
}

# Function to run integration tests
run_integration_tests() {
    print_header "Integration Tests"

    local scenarios=(
        "integration_scenarios::test_complete_multi_environment_setup"
        "integration_scenarios::test_template_based_development"
    )

    for scenario in "${scenarios[@]}"; do
        if [ -z "$TEST_FILTER" ] || [[ "$scenario" == *"$TEST_FILTER"* ]]; then
            echo -e "\n${YELLOW}Running $scenario...${NC}"
            cargo test --test cloud_commands_test "$scenario" -- --quiet || {
                print_error "Failed: $scenario"
                exit 1
            }
            print_success "Passed: $scenario"
        fi
    done
}

# Function to run error handling tests
run_error_tests() {
    print_header "Error Handling Tests"

    cargo test --test cloud_commands_test "error_handling_tests" -- --quiet || {
        print_error "Error handling tests failed"
        exit 1
    }

    print_success "Error handling tests passed"
}

# Function to run mock service tests
run_mock_tests() {
    print_header "Mock Service Tests"

    echo "Testing AWS ECR mocks..."
    cargo test --lib "mocks::aws" -- --quiet || {
        print_error "AWS mock tests failed"
        exit 1
    }
    print_success "AWS mocks OK"

    echo "Testing Fly.io mocks..."
    cargo test --lib "mocks::fly" -- --quiet || {
        print_error "Fly mock tests failed"
        exit 1
    }
    print_success "Fly mocks OK"

    echo "Testing Helix Cloud mocks..."
    cargo test --lib "mocks::helix_cloud" -- --quiet || {
        print_error "Helix Cloud mock tests failed"
        exit 1
    }
    print_success "Helix Cloud mocks OK"

    echo "Testing Docker mocks..."
    cargo test --lib "mocks::docker" -- --quiet || {
        print_error "Docker mock tests failed"
        exit 1
    }
    print_success "Docker mocks OK"
}

# Function to validate test plan coverage
validate_coverage() {
    print_header "Validating Test Coverage"

    local total_planned=$(grep -c "test_" cloud_commands_test_plan.md)
    local total_implemented=$(grep -c "#\[test\]" tests/cloud_commands_test.rs)

    echo "Test cases planned: $total_planned"
    echo "Test cases implemented: $total_implemented"

    if [ "$total_implemented" -lt "$total_planned" ]; then
        print_warning "Not all planned tests are implemented yet"
    else
        print_success "Test coverage matches plan"
    fi
}

# Function to run specific test suites
run_specific_suite() {
    case "$1" in
        init)
            run_test_suite "Init Command" "init_tests"
            ;;
        add)
            run_test_suite "Add Command" "add_tests"
            ;;
        push)
            run_test_suite "Push Command" "push_tests"
            ;;
        build)
            run_test_suite "Build Command" "build_tests"
            ;;
        auth)
            run_test_suite "Auth Command" "auth_tests"
            ;;
        config)
            run_test_suite "Configuration" "config_tests"
            ;;
        integration)
            run_integration_tests
            ;;
        error)
            run_error_tests
            ;;
        mocks)
            run_mock_tests
            ;;
        *)
            echo "Unknown test suite: $1"
            echo "Available suites: init, add, push, build, auth, config, integration, error, mocks"
            exit 1
            ;;
    esac
}

# Main test execution
main() {
    print_header "Helix CLI Cloud Commands Test Suite"

    # Check for Rust and Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed"
        exit 1
    fi

    # Build the project first
    echo "Building project..."
    cargo build --all-features || {
        print_error "Build failed"
        exit 1
    }
    print_success "Build successful"

    # Run tests based on arguments
    if [ -n "$TEST_FILTER" ] && [ "$TEST_FILTER" != "all" ]; then
        run_specific_suite "$TEST_FILTER"
    else
        # Run all test suites
        run_unit_tests
        run_integration_tests
        run_error_tests
        run_mock_tests
        validate_coverage
    fi

    # Generate coverage report if requested
    if [ "$COVERAGE" = "true" ]; then
        print_header "Coverage Report"
        echo "Coverage report generated at: target/coverage/index.html"

        if command -v open &> /dev/null; then
            open target/coverage/index.html
        elif command -v xdg-open &> /dev/null; then
            xdg-open target/coverage/index.html
        fi
    fi

    print_header "Test Summary"
    print_success "All tests passed successfully!"
}

# Handle command line arguments
show_help() {
    cat << EOF
Usage: $0 [TEST_SUITE] [OPTIONS]

Run comprehensive tests for Helix CLI cloud commands.

TEST_SUITE:
    all         Run all test suites (default)
    init        Test init command
    add         Test add command
    push        Test push command
    build       Test build command
    auth        Test auth commands
    config      Test configuration
    integration Run integration tests
    error       Run error handling tests
    mocks       Test mock services

OPTIONS:
    VERBOSE=true      Show detailed output
    PARALLEL=false    Run tests sequentially
    COVERAGE=true     Generate code coverage report

Examples:
    $0                    # Run all tests
    $0 init              # Run only init command tests
    $0 integration       # Run integration tests
    VERBOSE=true $0      # Run all tests with verbose output
    COVERAGE=true $0     # Run tests and generate coverage report
EOF
}

# Parse arguments
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    show_help
    exit 0
fi

# Run main function
main