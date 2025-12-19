// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

use kube_condition::StatusCondition;
use thiserror::Error;

#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
pub enum CustomError {
    #[error("retryable error: {0}")]
    #[condition(reason = "RetryableFailure", severity = "warning", retryable = true, requeue_secs = 60)]
    Retryable(String),

    #[error("non-retryable error: {0}")]
    #[condition(reason = "PermanentFailure", severity = "error", retryable = false)]
    NonRetryable(String),

    #[error("info level error")]
    #[condition(reason = "MinorIssue", severity = "info", retryable = true, requeue_secs = 10)]
    InfoLevel,
}

fn main() {}
