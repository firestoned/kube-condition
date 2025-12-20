// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

//! Compile-time tests for the StatusCondition derive macro using trybuild

#[test]
fn test_compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}

// TODO: Add compile-fail tests when we have error cases to test
// #[test]
// fn test_compile_fail() {
//     let t = trybuild::TestCases::new();
//     t.compile_fail("tests/fail/*.rs");
// }
