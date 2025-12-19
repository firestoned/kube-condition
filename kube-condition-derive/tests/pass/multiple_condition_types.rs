// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

use kube_condition::StatusCondition;
use thiserror::Error;

#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
pub enum MultiTypeError {
    #[error("ready condition error")]
    #[condition(condition_type = "Ready", reason = "NotReady")]
    ReadyError,

    #[error("synchronized condition error")]
    #[condition(condition_type = "Synchronized", reason = "SyncFailed")]
    SyncError,

    #[error("degraded condition error")]
    #[condition(condition_type = "Degraded", reason = "PerformanceDegraded", status = "True")]
    DegradedError,
}

fn main() {}
