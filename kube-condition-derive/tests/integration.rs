// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

//! Integration tests for StatusCondition derive macro

use kube_condition::{
    Severity, StatusCondition, CONDITION_STATUS_FALSE, CONDITION_STATUS_TRUE, CONDITION_TYPE_READY,
};
use std::time::Duration;
use thiserror::Error;

/// Test basic derive with minimal attributes
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
enum BasicError {
    #[error("simple error")]
    #[condition(reason = "SimpleFailure")]
    Simple,
}

#[test]
fn test_basic_derive() {
    let error = BasicError::Simple;
    let info = error.to_condition_info();

    assert_eq!(info.type_, CONDITION_TYPE_READY);
    assert_eq!(info.status, CONDITION_STATUS_FALSE);
    assert_eq!(info.reason, "SimpleFailure");
    assert_eq!(info.message, "simple error");

    assert_eq!(error.severity(), Severity::Error); // Default
    assert!(error.is_retryable()); // Default
    assert_eq!(error.requeue_duration(), Duration::from_secs(30)); // Default
}

/// Test custom severity levels
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
enum SeverityError {
    #[error("info error")]
    #[condition(reason = "InfoError", severity = "info")]
    Info,

    #[error("warning error")]
    #[condition(reason = "WarningError", severity = "warning")]
    Warning,

    #[error("error error")]
    #[condition(reason = "ErrorError", severity = "error")]
    Error,
}

#[test]
fn test_severity_levels() {
    assert_eq!(SeverityError::Info.severity(), Severity::Info);
    assert_eq!(SeverityError::Warning.severity(), Severity::Warning);
    assert_eq!(SeverityError::Error.severity(), Severity::Error);
}

/// Test retryable attribute
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
enum RetryError {
    #[error("retryable")]
    #[condition(reason = "Retryable", retryable = true, requeue_secs = 60)]
    Retryable,

    #[error("non-retryable")]
    #[condition(reason = "NonRetryable", retryable = false)]
    NonRetryable,
}

#[test]
fn test_retryable() {
    let retryable = RetryError::Retryable;
    assert!(retryable.is_retryable());
    assert_eq!(retryable.requeue_duration(), Duration::from_secs(60));

    let non_retryable = RetryError::NonRetryable;
    assert!(!non_retryable.is_retryable());
    // Non-retryable still has a requeue duration (defaults to 30)
    assert_eq!(non_retryable.requeue_duration(), Duration::from_secs(30));
}

/// Test different default_type values
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Synchronized")]
enum SyncError {
    #[error("sync failed")]
    #[condition(reason = "SyncFailed")]
    SyncFailed,

    #[error("degraded error")]
    #[condition(reason = "Degraded", status = "True")]
    Degraded,
}

#[test]
fn test_custom_default_type() {
    let sync = SyncError::SyncFailed;
    assert_eq!(sync.to_condition_info().type_, "Synchronized");
    assert_eq!(sync.to_condition_info().status, CONDITION_STATUS_FALSE);

    let degraded = SyncError::Degraded;
    assert_eq!(degraded.to_condition_info().type_, "Synchronized");
    assert_eq!(degraded.to_condition_info().status, CONDITION_STATUS_TRUE);
}

/// Test with enum variants containing data
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
enum DataError {
    #[error("error with string: {0}")]
    #[condition(reason = "StringError")]
    WithString(String),

    #[error("error with fields: {message}, code: {code}")]
    #[condition(reason = "FieldError")]
    WithFields { message: String, code: i32 },
}

#[test]
fn test_enum_with_data() {
    let string_error = DataError::WithString("test message".to_string());
    let info = string_error.to_condition_info();
    assert_eq!(info.reason, "StringError");
    assert_eq!(info.message, "error with string: test message");

    let field_error = DataError::WithFields {
        message: "field message".to_string(),
        code: 42,
    };
    let info = field_error.to_condition_info();
    assert_eq!(info.reason, "FieldError");
    assert_eq!(info.message, "error with fields: field message, code: 42");
}

/// Test that default_type is used when type is not specified
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Available")]
enum DefaultTypeError {
    #[error("uses default type")]
    #[condition(reason = "UsesDefault")]
    UsesDefault,
}

#[test]
fn test_default_type() {
    let error = DefaultTypeError::UsesDefault;
    assert_eq!(error.to_condition_info().type_, "Available");
}

/// Test to_condition() method (trait default implementation)
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
enum ConditionError {
    #[error("test")]
    #[condition(reason = "Test")]
    Test,
}

#[test]
fn test_to_condition() {
    let error = ConditionError::Test;
    let condition = error.to_condition(Some(5));

    assert_eq!(condition.type_, CONDITION_TYPE_READY);
    assert_eq!(condition.status, CONDITION_STATUS_FALSE);
    assert_eq!(condition.reason, "Test");
    assert_eq!(condition.message, "test");
    assert_eq!(condition.observed_generation, Some(5));
    assert!(condition.last_transition_time.0 <= k8s_openapi::chrono::Utc::now());
}

/// Test to_action() method (trait default implementation)
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
enum ActionError {
    #[error("retryable action")]
    #[condition(reason = "Retryable", retryable = true, requeue_secs = 45)]
    Retryable,

    #[error("non-retryable action")]
    #[condition(reason = "NonRetryable", retryable = false)]
    NonRetryable,
}

#[test]
fn test_to_action() {
    let retryable = ActionError::Retryable;
    let action = retryable.to_action();
    // We can't directly inspect Action, but we can verify it compiles
    let _ = action;
    assert!(retryable.is_retryable());
    assert_eq!(retryable.requeue_duration(), Duration::from_secs(45));

    let non_retryable = ActionError::NonRetryable;
    let action = non_retryable.to_action();
    let _ = action;
    assert!(!non_retryable.is_retryable());
}
