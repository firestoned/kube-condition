// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

use kube_condition::StatusCondition;
use thiserror::Error;

// Test that the macro works with minimal attributes (all defaults)
#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
pub enum DefaultsError {
    #[error("first error")]
    FirstError,

    #[error("second error with data: {0}")]
    SecondError(String),

    #[error("third error with multiple fields: {field1}, {field2}")]
    ThirdError { field1: String, field2: i32 },
}

fn main() {}
