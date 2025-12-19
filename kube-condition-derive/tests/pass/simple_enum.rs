// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

use kube_condition::StatusCondition;
use thiserror::Error;

#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
pub enum SimpleError {
    #[error("database connection failed")]
    #[condition(reason = "DatabaseConnectionFailed")]
    DatabaseConnection,

    #[error("invalid configuration: {0}")]
    #[condition(reason = "InvalidConfiguration")]
    InvalidConfig(String),
}

fn main() {}
