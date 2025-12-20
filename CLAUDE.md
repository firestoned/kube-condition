# Project Instructions for Claude Code

> Platform Engineering - Kubernetes Operators & Infrastructure
> Environment: k0rdent / Capital Markets / Multi-cluster
>
> **Service Mesh Standard**: Always use Linkerd as the example service mesh in documentation, examples, and code comments. Do not use generic "service mesh" references or other mesh implementations (Istio, Consul Connect, etc.) unless specifically required.

---

## Project Overview: kube-condition

**kube-condition** is a Rust library for mapping errors to Kubernetes status conditions in Kubernetes operators. It consists of two main components:

1. **kube-condition-derive** - A procedural macro crate providing `#[derive(StatusCondition)]`
2. **kube-condition** - Runtime library with traits, helpers, and reconcile wrappers

### Key Features

- **Declarative Error Mapping**: Define Kubernetes conditions directly on error enums using attributes
- **Automatic Status Updates**: Errors automatically generate appropriate Kubernetes conditions
- **Retry Control**: Configure retry behavior per error variant (retryable, requeue duration)
- **Severity Levels**: Map errors to different severity levels (info/warning/error) for observability
- **Type Safety**: Compile-time guarantees for condition mappings
- **Single Source of Truth**: Error behavior defined alongside error types

### Architecture

```
kube-condition/
├── kube-condition-derive/     # Proc macro crate
│   ├── Cargo.toml
│   └── src/lib.rs            # #[derive(StatusCondition)] implementation
├── kube-condition/            # Runtime library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # StatusCondition trait and types
│       ├── condition.rs      # Condition building helpers
│       └── reconcile.rs      # reconcile_with_status! macro
└── examples/                  # Example operators using the library
    └── dns-operator/
```

---

## 🚨 Critical TODOs

### Code Quality: Use Global Constants for Repeated Strings
**Status:** 🔄 Ongoing
**Impact:** Code maintainability and consistency

When a string literal appears in multiple places across the codebase, it MUST be defined as a global constant and referenced consistently.

**Why:**
- **Single Source of Truth**: Changes only need to be made in one place
- **Consistency**: Prevents typos and inconsistencies across the codebase
- **Maintainability**: Easier to refactor and update values
- **Type Safety**: Compiler catches usage errors

**When to Create a Global Constant:**
- String appears 2+ times in the same file
- String appears in multiple files
- String represents a configuration value (condition types, reasons, status values, etc.)
- String is part of an API contract or protocol

**Examples:**
```rust
// ✅ GOOD - Use constants
const CONDITION_TYPE_READY: &str = "Ready";
const CONDITION_TYPE_DNSSEC_READY: &str = "DnssecReady";
const CONDITION_STATUS_TRUE: &str = "True";
const CONDITION_STATUS_FALSE: &str = "False";

impl StatusCondition for MyError {
    fn to_condition_info(&self) -> ConditionInfo {
        ConditionInfo {
            type_: CONDITION_TYPE_READY.to_string(),
            status: CONDITION_STATUS_FALSE.to_string(),
            ..
        }
    }
}

// ❌ BAD - Hardcoded strings
impl StatusCondition for MyError {
    fn to_condition_info(&self) -> ConditionInfo {
        ConditionInfo {
            type_: "Ready".to_string(),
            status: "False".to_string(),
            ..
        }
    }
}
```

**Where to Define Constants:**
- Module-level constants: At the top of the file for file-specific use
- Crate-level constants: In a dedicated module (e.g., `src/constants.rs`) for cross-module use
- Group related constants together with documentation

**Verification:**
Before committing, search for repeated string literals:
```bash
# Find potential duplicate strings in Rust files
grep -rn '"[^"]\{5,\}"' src/ | sort | uniq -d
```

---

## 🔧 GitHub Workflows & CI/CD

### CRITICAL: All Workflows Must Be Makefile-Driven

**Status:** ✅ Required Standard
**Impact:** Consistency, maintainability, and local reproducibility

All GitHub Actions workflows MUST delegate complex logic to Makefile targets. Workflows should only:
1. Install required tools (Rust, cargo-expand, etc.)
2. Set up environment variables
3. Call Makefile targets

**Why:**
- **Local Reproducibility**: Developers can run the exact same commands locally
- **Consistency**: Same logic runs in CI and locally
- **Maintainability**: Business logic lives in one place (Makefile), not scattered across workflows
- **Testability**: Makefile targets can be tested independently
- **Simplicity**: Workflows become declarative configuration, not complex scripts

**Pattern:**

```yaml
# ✅ GOOD - Workflow delegates to Makefile
jobs:
  test:
    steps:
      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run tests
        run: make test

      - name: Run macro expansion tests
        run: make test-expand

# ❌ BAD - Complex logic in workflow
jobs:
  test:
    steps:
      - name: Run tests
        run: |
          cargo test --lib
          cargo test --doc
          cargo expand --lib > expanded.rs
          # ... 50+ lines of bash ...
```

**Requirements:**
- Workflows MUST NOT contain multi-line bash scripts (except simple tool setup)
- All test orchestration MUST be in Makefile targets
- All build logic MUST be in Makefile targets
- Makefile targets MUST work identically locally and in CI
- Document Makefile targets with `## comments` for `make help`

### CRITICAL: Workflows Must Be Reusable and Composable

**Status:** ✅ Required Standard
**Impact:** Maintainability, DRY principles, and workflow consistency

When adding new GitHub Actions workflows, they MUST be designed for reusability and integration with existing workflows.

**Why:**
- **DRY Principle**: Avoid duplicating workflow logic across multiple files
- **Consistency**: Same steps produce same results across different contexts
- **Maintainability**: Update shared logic once, not in every workflow
- **Composability**: Workflows can call other workflows or be called by others
- **Flexibility**: Standalone execution and integration into larger workflows

**Requirements:**

1. **Use Reusable Workflows** (`.github/workflows/*.yml` with `workflow_call`):
   - Define workflows that can be called by other workflows
   - Accept inputs for configuration
   - Define outputs for downstream steps
   - Make them standalone executable (support both `workflow_call` and manual triggers)

2. **Use Composite Actions** (`.github/actions/*/action.yml`):
   - For complex multi-step operations that are used across multiple workflows
   - For shared setup/teardown logic
   - For operations that need to be consistent across workflows

3. **Integration Strategy**:
   - New workflows MUST be callable from existing workflows
   - Existing workflows SHOULD be able to include new workflow steps
   - Avoid creating isolated workflows that duplicate existing logic

**Checklist for New Workflows:**

Before adding a new workflow, ask:
- [ ] Can this be added as a job to an existing workflow?
- [ ] Can this be made into a reusable workflow that others can call?
- [ ] Does this duplicate logic from an existing workflow?
- [ ] Can this be extracted into a composite action for reuse?
- [ ] Will existing workflows benefit from calling this workflow?
- [ ] Can this workflow be triggered both standalone and as a called workflow?

---

## 🔒 Compliance & Security Context

**CRITICAL: This codebase operates in a highly regulated banking environment and MUST comply with:**

### Regulatory Frameworks
- **NIST** (National Institute of Standards and Technology)
  - NIST Cybersecurity Framework (CSF)
  - NIST SP 800-53 (Security and Privacy Controls)
  - NIST SP 800-171 (Protecting Controlled Unclassified Information)
- **FIPS** (Federal Information Processing Standards)
  - FIPS 140-2/140-3 for cryptographic modules
  - Only FIPS-approved cryptographic algorithms
- **Basel III** (Banking Regulations)
  - Operational risk management requirements
  - Technology and cyber resilience principles
- **SOX** (Sarbanes-Oxley Act)
  - IT general controls (ITGCs)
  - Change management controls
  - Segregation of duties

### Security Requirements

All changes must be:
- **Auditable**: Clear documentation and attribution for every change
- **Traceable**: Linked to a business or technical requirement
- **Zero-trust compliant**: Never assume trust, always verify
- **Cryptographically sound**: Only FIPS-approved algorithms (AES, SHA-256, RSA, etc.)
- **Immutable and tamper-evident**: Changes must be logged and traceable

### Code Security Standards

**CRITICAL - Never commit**:
- Secrets, tokens, API keys, or credentials (even examples or placeholders)
- Internal hostnames, IP addresses, or network topology information
- Customer data, PII, or transaction data in any form
- Hardcoded passwords or encryption keys
- Test data that resembles real customer data

**CRITICAL - Always implement**:
- Input validation at all system boundaries
- Proper error handling without exposing sensitive information
- Secure defaults (fail closed, not open)
- Defense in depth (multiple layers of security)
- Principle of least privilege

### Cryptography Requirements

**FIPS 140-2/140-3 Compliance**:
- Use only FIPS-approved cryptographic algorithms:
  - **Encryption**: AES (128/192/256-bit), 3DES (legacy systems only)
  - **Hashing**: SHA-256, SHA-384, SHA-512 (NOT MD5 or SHA-1)
  - **Key Exchange**: RSA (2048-bit minimum), ECDH (P-256, P-384, P-521)
  - **Signatures**: RSA-PSS, ECDSA (P-256, P-384, P-521)
- Document cryptographic operations in ADRs
- Never implement custom cryptography - use vetted libraries (e.g., `ring`, `rustls`)
- Key management must follow NIST SP 800-57 guidelines

### Change Management (SOX Compliance)

**MANDATORY for ALL code changes**:
- **Author attribution**: Every changelog entry must identify the author
- **Justification**: Document the business or technical reason for the change
- **Testing evidence**: All tests must pass before deployment
- **Peer review**: Code changes require review (use PRs)
- **Audit trail**: Git history must be preserved and signed

### Operational Risk Management (Basel III)

**Technology Resilience Requirements**:
- **Availability**: Design for high availability and disaster recovery
- **Observability**: Comprehensive logging and monitoring (no PII in logs)
- **Incident Response**: Document error conditions and recovery procedures
- **Dependency Management**: Vet all third-party dependencies for security vulnerabilities
- **Change Impact**: Document potential impact of changes on system stability

### Segregation of Duties

- No single person should have complete control over critical operations
- Code review required before merging changes
- Production deployments require approval from authorized personnel
- Administrative access must be logged and auditable

---

**Violation of these compliance requirements may result in regulatory penalties, audit findings, or security incidents. When in doubt, consult the security or compliance team.**

---

## 📝 Documentation Requirements

### Mandatory: Documentation Updates for Code Changes

**CRITICAL: After ANY code change in the `src/` directory, you MUST update all relevant documentation.**

This is a **mandatory step** that must be completed before considering any task complete. Documentation must always reflect the current state of the code.

#### Documentation Update Workflow

When adding, removing, or changing any feature in the Rust source code:

1. **Analyze the Change**:
   - What functionality was added/removed/changed?
   - What are the user-facing impacts?
   - What are the API changes?
   - Are there new macros, attributes, or behaviors?

2. **Update Documentation** (in this order):
   - **`CHANGELOG.md`** - Document the change (see format below)
   - **`README.md`** - Update if API or getting started changed
   - **`docs/`** - Update all affected documentation pages:
     - User guides that reference the changed functionality
     - Quickstart guides with examples of the changed code
     - Attribute reference for macro changes
     - Troubleshooting guides if behavior changed
   - **`examples/`** - Update example code to reflect changes
   - **API documentation** - Ensure rustdoc comments are accurate

3. **Verify Documentation Accuracy**:
   - Read through updated docs as if you're a new user
   - Ensure all code examples compile and run
   - Verify all examples demonstrate correct usage
   - Check that attribute documentation matches macro implementation
   - Confirm API docs reflect current trait signatures

4. **Add Missing Documentation**:
   - If API changed, add/update API documentation
   - If new attributes were added, document them with examples
   - If new traits exist, document them
   - If new error conditions exist, document troubleshooting steps
   - If new dependencies were added, document version requirements

#### What Documentation to Update

**For Macro Changes** (`kube-condition-derive/src/lib.rs`):
- Update attribute reference documentation
- Document new attributes with examples
- Update examples showing macro usage
- Update quickstart guides
- Regenerate macro expansion examples

**For Runtime Library Changes** (`kube-condition/src/`):
- Update trait documentation
- Add code examples for new public functions
- Update troubleshooting guides for new behaviors
- Document new types and their purpose

**For New Features**:
- Add feature documentation to `/docs/`
- Update feature list in README.md
- Add usage examples
- Document configuration options
- Add troubleshooting section

**For Bug Fixes**:
- Update troubleshooting guides with the fix
- Document workarounds (if applicable) in known issues
- Update behavior documentation if expectations changed

#### Documentation Quality Standards

- **Completeness**: All user-visible changes must be documented
- **Accuracy**: Documentation must match the actual code behavior
- **Examples**: Include working examples for all features
- **Clarity**: Write for users who haven't seen the code
- **Versioning**: Date all changes in CHANGELOG.md

#### Building Documentation

```bash
# Build rustdoc for all crates
cargo doc --all --no-deps --open

# Build user documentation (if using mdbook)
mdbook build docs
```

#### Validation Checklist

Before considering a task complete, verify:
- [ ] CHANGELOG.md updated with change details
- [ ] All affected documentation pages updated
- [ ] All code examples compile and run
- [ ] API documentation regenerated
- [ ] README.md updated (if API or features changed)
- [ ] No broken links in documentation
- [ ] Documentation reviewed as if reading for the first time

### Mandatory: Update Changelog on Every Code Change

After **ANY** code modification, update `CHANGELOG.md` with the following format:

```markdown
## [YYYY-MM-DD HH:MM] - Brief Title

**Author:** [Author Name]

### Changed
- `path/to/file.rs`: Description of the change

### Why
Brief explanation of the business or technical reason.

### Impact
- [ ] Breaking change
- [ ] New feature
- [ ] Bug fix
- [ ] Documentation only
```

**CRITICAL REQUIREMENT**:
- The `**Author:**` line is **MANDATORY** for ALL changelog entries
- This is required for auditing and accountability in a regulated environment
- The author field should contain the name of the person who requested or approved the change
- **NO exceptions** - every changelog entry must have an author attribution
- If the author is unknown, use "Unknown" but investigate to identify the proper author

### Code Comments

All public functions, types, and macros **must** have rustdoc comments:

```rust
/// Trait for errors that can be mapped to Kubernetes status conditions.
///
/// Implement this trait (or derive it with `#[derive(StatusCondition)]`) to
/// automatically convert errors into Kubernetes condition objects.
///
/// # Example
///
/// ```rust
/// use kube_condition::StatusCondition;
/// use thiserror::Error;
///
/// #[derive(Error, Debug, StatusCondition)]
/// #[condition(default_type = "Ready")]
/// pub enum MyError {
///     #[error("Failed to connect: {0}")]
///     #[condition(reason = "ConnectionFailed", retryable = true)]
///     Connection(String),
/// }
/// ```
pub trait StatusCondition {
    /// Converts the error into condition information.
    fn to_condition_info(&self) -> ConditionInfo;

    /// Returns the severity level for logging/metrics.
    fn severity(&self) -> Severity;

    /// Returns whether the error is retryable.
    fn is_retryable(&self) -> bool;

    /// Returns the duration to wait before retry.
    fn requeue_duration(&self) -> Duration;
}
```

### Architecture Decision Records (ADRs)

For significant design decisions, create `/docs/adr/NNNN-title.md`:

```markdown
# ADR-NNNN: Title

## Status
Proposed | Accepted | Deprecated | Superseded by ADR-XXXX

## Context
What is the issue we're facing?

## Decision
What have we decided to do?

## Consequences
What are the trade-offs?
```

---

## 🦀 Rust Workflow

### After Modifying Any `.rs` File

**CRITICAL: At the end of EVERY task that modifies Rust files, ALWAYS run these commands in order:**

```bash
# 1. Format code
cargo fmt

# 2. Run clippy with strict warnings
cargo clippy --all -- -D warnings -W clippy::pedantic -A clippy::module_name_repetitions

# 3. Run tests (all crates)
cargo test --all

# 4. Check for security vulnerabilities (if cargo-audit installed)
cargo audit 2>/dev/null || true
```

**IMPORTANT:**
- This is MANDATORY at the end of every task involving Rust code changes
- Fix ALL clippy warnings before considering the task complete
- Do NOT skip these steps - they catch bugs and ensure code quality
- If clippy or tests fail, the task is NOT complete

**CRITICAL: After ANY Rust code modification, you MUST verify:**

1. **Function documentation is accurate**:
   - Check rustdoc comments match what the function actually does
   - Verify all `# Arguments` match the actual parameters
   - Verify `# Returns` matches the actual return type
   - Verify `# Errors` describes all error cases
   - Update examples in doc comments if behavior changed

2. **Unit tests are accurate and passing**:
   - Check test assertions match the new behavior
   - Update test expectations if behavior changed
   - Ensure all tests compile and run successfully
   - Add new tests for new behavior/edge cases

3. **Documentation is updated**:
   - Update relevant files in `docs/` directory
   - Update examples in `examples/` directory
   - Ensure `CHANGELOG.md` reflects the changes
   - Verify example code compiles successfully

### Unit Testing Requirements

**CRITICAL: When modifying ANY Rust code, you MUST update, add, or delete unit tests accordingly:**

1. **Adding New Functions/Methods:**
   - MUST add unit tests for ALL new public functions
   - Test both success and failure scenarios
   - Include edge cases and boundary conditions

2. **Modifying Existing Functions:**
   - MUST update existing tests to reflect changes
   - Add new tests if new behavior or code paths are introduced
   - Ensure ALL existing tests still pass

3. **Deleting Functions:**
   - MUST delete corresponding unit tests
   - Remove or update integration tests that depended on deleted code

4. **Refactoring Code:**
   - Update test names and assertions to match refactored code
   - Verify test coverage remains the same or improves
   - If refactoring changes function signatures, update ALL tests

5. **Test Quality Standards:**
   - Use descriptive test names (e.g., `test_status_condition_generates_ready_false`)
   - Follow the Arrange-Act-Assert pattern
   - Mock external dependencies (k8s API, external services)
   - Test error conditions, not just happy paths
   - Ensure tests are deterministic (no flaky tests)

6. **Test File Organization:**
   - **CRITICAL**: Place tests in `#[cfg(test)] mod tests {}` blocks at the end of files
   - For proc macros, use integration tests in `tests/` directory
   - Test macro expansion with `cargo expand` or trybuild

**VERIFICATION:**
- After ANY Rust code change, run `cargo test --all`
- ALL tests MUST pass before the task is considered complete
- If you add code but cannot write a test, document WHY in the code comments

**Example:**
If you modify `kube-condition/src/lib.rs`:
1. Update/add tests in the `tests` module
2. Run `cargo test -p kube-condition` to verify
3. Ensure ALL tests pass before moving on

For proc macro changes in `kube-condition-derive/`:
1. Add integration tests in `tests/` directory
2. Use `cargo expand` to verify generated code
3. Run `cargo test -p kube-condition-derive`

### Rust Style Guidelines

- Use `thiserror` for error types, not string errors
- Prefer `anyhow::Result` in binaries, typed errors in libraries
- Use `tracing` for logging, not `println!` or `log`
- Async functions should use `tokio`
- **No magic numbers**: Any numeric literal other than `0` or `1` MUST be declared as a named constant
- **Use early returns/guard clauses**: Minimize nesting by handling edge cases early and returning

#### Early Return / Guard Clause Pattern

**CRITICAL: Prefer early returns over nested if-else statements.**

The "early return" or "guard clause" coding style emphasizes minimizing nested if-else statements and promoting clearer, more linear code flow.

**Key Principles:**

1. **Handle preconditions first**: Validate input parameters and other preconditions at the start of a function.

   ```rust
   // ✅ GOOD - Early return for validation
   pub fn to_condition_info(&self) -> ConditionInfo {
       // Guard clause: Check if we need custom handling
       if let Some(custom_type) = self.custom_type() {
           return ConditionInfo {
               type_: custom_type,
               ..Default::default()
           };
       }

       // Main logic continues here (happy path)
       ConditionInfo {
           type_: DEFAULT_CONDITION_TYPE.to_string(),
           ..Default::default()
       }
   }

   // ❌ BAD - Nested if-else
   pub fn to_condition_info(&self) -> ConditionInfo {
       if let Some(custom_type) = self.custom_type() {
           ConditionInfo {
               type_: custom_type,
               ..Default::default()
           }
       } else {
           ConditionInfo {
               type_: DEFAULT_CONDITION_TYPE.to_string(),
               ..Default::default()
           }
       }
   }
   ```

2. **Use `?` for error propagation**: Rust's `?` operator is a form of early return for errors.

   ```rust
   // ✅ GOOD - Early error returns with ?
   pub fn parse_attribute(attr: &Attribute) -> Result<ConditionAttr> {
       let meta = attr.parse_meta()?;
       let list = match meta {
           Meta::List(list) => list,
           _ => return Err(Error::new_spanned(attr, "Expected attribute list")),
       };

       Ok(ConditionAttr::from_list(list)?)
   }
   ```

**Benefits:**
- **Reduced nesting**: Improves readability and reduces cognitive load
- **Clearer code flow**: The main logic is less cluttered by error handling
- **Easier to test**: Each condition can be tested in isolation
- **Fail-fast approach**: Catches invalid states or inputs early

#### Magic Numbers Rule

**CRITICAL: Eliminate all magic numbers from the codebase.**

A "magic number" is any numeric literal (other than `0` or `1`) that appears directly in code without explanation.

**Rules:**
- **`0` and `1` are allowed** - These are ubiquitous and self-explanatory
- **All other numbers MUST be named constants** - No exceptions
- Use descriptive names that explain the *purpose*, not just the value

**Examples:**

```rust
// ✅ GOOD - Named constants
const DEFAULT_REQUEUE_SECONDS: u64 = 30;
const DEFAULT_RETRY_BACKOFF_SECONDS: u64 = 15;
const MAX_CONDITION_MESSAGE_LENGTH: usize = 256;

impl StatusCondition for MyError {
    fn requeue_duration(&self) -> Duration {
        Duration::from_secs(DEFAULT_REQUEUE_SECONDS)
    }
}

// ❌ BAD - Magic numbers
impl StatusCondition for MyError {
    fn requeue_duration(&self) -> Duration {
        Duration::from_secs(30)  // Why 30? What does it mean?
    }
}
```

**Where to Define Constants:**
- Module-level: For constants used only within one file
- Crate-level (`src/constants.rs`): For constants used across modules
- Group related constants together with documentation

**Test Files Exception:**
Test files may use literal values for test data when it improves readability and the values are only used once.

### Dependency Management

Before adding a new dependency:
1. Check if existing deps solve the problem
2. Verify the crate is actively maintained (commits in last 6 months)
3. Prefer crates from well-known authors or the Rust ecosystem
4. Document why the dependency was added in `CHANGELOG.md`

---

## 🧪 Testing Requirements

### Unit Tests

**MANDATORY: Every public function MUST have corresponding unit tests.**

#### Test File Organization

Tests should be organized in `#[cfg(test)] mod tests {}` blocks at the end of source files.

For proc macros, use integration tests in the `tests/` directory to verify generated code.

**Test Coverage Requirements:**
- **Success path:** Test the primary expected behavior
- **Failure paths:** Test error handling for each possible error type
- **Edge cases:** Empty strings, null values, boundary conditions
- **Macro expansion:** For proc macros, verify generated code is correct

**When to Update Tests:**
- **ALWAYS** when adding new functions → Add new tests
- **ALWAYS** when modifying functions → Update existing tests
- **ALWAYS** when deleting functions → Delete corresponding tests
- **ALWAYS** when refactoring → Verify tests still cover the same behavior

### Integration Tests

For proc macros:
- Place integration tests in `/tests/` directory
- Use `cargo expand` to verify macro expansion
- Use `trybuild` for compile-fail tests
- Test both successful and failing macro usage

### Test Execution

**Before committing ANY Rust changes:**
```bash
# Run all tests (all crates)
cargo test --all

# Run tests for a specific crate
cargo test -p kube-condition
cargo test -p kube-condition-derive

# Run tests with output
cargo test -- --nocapture

# Verify macro expansion
cargo expand --lib kube-condition-derive
```

**ALL tests MUST pass before code is considered complete.**

---

## 📁 File Organization

```
kube-condition/
├── Cargo.toml                      # Workspace definition
├── README.md                       # Main project README
├── CHANGELOG.md                    # Mandatory changelog
├── LICENSE                         # MIT license
├── .github/
│   └── workflows/
│       ├── ci.yml                  # Main CI workflow
│       └── release.yml             # Release automation
├── kube-condition-derive/          # Proc macro crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                  # Derive macro implementation
├── kube-condition/                 # Runtime library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Public API and StatusCondition trait
│       ├── condition.rs            # ConditionInfo and helpers
│       ├── reconcile.rs            # reconcile_with_status! macro
│       └── severity.rs             # Severity enum
├── examples/                       # Example operators
│   └── dns-operator/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── error.rs            # Error types using #[derive(StatusCondition)]
│           └── controller.rs       # Reconcile loop using the library
└── docs/                           # Documentation (if using mdbook)
    ├── book.toml
    └── src/
        ├── SUMMARY.md
        ├── introduction.md
        └── guide/
```

---

## 🚫 Things to Avoid

- **Never** use `unwrap()` in library code - use `?` or explicit error handling
- **Never** use `panic!()` in library code - return proper errors
- **Never** ignore errors in generated code - always propagate them properly
- **Never** make breaking API changes without documenting migration path
- **Never** commit generated code that doesn't compile

---

## 💡 Helpful Commands

```bash
# Format all code
cargo fmt --all

# Run clippy on all crates
cargo clippy --all -- -D warnings

# Run all tests
cargo test --all

# Verify macro expansion
cargo expand --lib kube-condition-derive

# Build documentation
cargo doc --all --no-deps --open

# Publish to crates.io (release)
cargo publish -p kube-condition-derive
cargo publish -p kube-condition
```

---

## 📋 PR/Commit Checklist

**MANDATORY: Run this checklist at the end of EVERY task before considering it complete.**

Before committing:

- [ ] **If ANY `.rs` file was modified**:
  - [ ] **Unit tests updated/added/deleted** to match code changes (REQUIRED)
  - [ ] All new public functions have corresponding tests (REQUIRED)
  - [ ] All modified functions have updated tests (REQUIRED)
  - [ ] All deleted functions have tests removed (REQUIRED)
  - [ ] `cargo fmt --all` passes (REQUIRED)
  - [ ] `cargo clippy --all -- -D warnings` passes (REQUIRED - fix ALL warnings)
  - [ ] `cargo test --all` passes (REQUIRED - ALL tests must pass)
  - [ ] **Documentation updated** for code changes (REQUIRED):
    - [ ] Rustdoc comments on ALL public items (functions, types, traits, macros)
    - [ ] Function documentation matches actual behavior (parameters, returns, errors)
    - [ ] Examples in documentation compile and run
    - [ ] README.md updated for user-facing changes
- [ ] **If proc macro code was modified** (`kube-condition-derive/`):
  - [ ] Integration tests added/updated in `tests/` directory
  - [ ] Macro expansion verified with `cargo expand`
  - [ ] Attribute documentation updated
  - [ ] Error messages are clear and helpful
- [ ] **Documentation verification** (CRITICAL):
  - [ ] `CHANGELOG.md` updated with detailed change description **AND author attribution** (REQUIRED)
  - [ ] Author name included in changelog entry (e.g., `**Author:** Erick Bourgeois`)
  - [ ] All affected documentation pages reviewed and updated
  - [ ] Code examples in docs compile and run
  - [ ] README.md updated if API or features changed
  - [ ] No broken links in documentation
- [ ] No secrets or sensitive data
- [ ] Error handling uses proper types (no `.unwrap()` or `.panic!()`)
- [ ] All examples compile: `cargo build --examples`

**A task is NOT complete until all of the above items pass successfully.**

**Documentation is NOT optional** - it is a critical requirement equal in importance to the code itself.

---

## 🔗 Project References

- [kube-rs documentation](https://kube.rs/)
- [Kubernetes API conventions](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md)
- [The Rust Programming Language - Procedural Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [syn crate documentation](https://docs.rs/syn/)
- [quote crate documentation](https://docs.rs/quote/)
- Internal: k0rdent platform docs (check Confluence)
