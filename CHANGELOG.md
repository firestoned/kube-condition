# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2025-12-21 19:00] - Add Version Requirements for Crates.io Publishing

**Author:** Erick Bourgeois

### Changed
- `kube-condition/Cargo.toml`: Added `version = "0.1.0"` to `kube-condition-derive` dependency to support crates.io publishing
- `kube-condition-derive/Cargo.toml`: Added `version = "0.1.0"` to `kube-condition` dev-dependency (required for integration tests)
- `.github/workflows/release.yaml`:
  - **MAJOR**: Restructured workflow from matrix-based to sequential job chain to handle dependency publishing order
  - New job dependency chain: `package-derive` → `sign-derive` → `publish-derive` → `package-main` → `sign-main` → `publish-main`
  - This ensures `kube-condition-derive` is published to crates.io BEFORE `kube-condition` is packaged (avoiding dependency resolution errors)
  - Added automatic version update step for `kube-condition-derive` dependency in packaging and publishing jobs
  - Updated all `firestoned/github-actions` references from `v1.2.4` to `v1.3.0`
  - Migrated to new composite actions: `rust/publish-crate@v1.3.0` and `rust/package-crate@v1.3.0`
  - Added `--no-verify` to `package-derive` step to skip verification (dev-dependency on unpublished crate)
  - 60-second wait after publishing `kube-condition-derive` for crates.io indexing

### Why
When publishing to crates.io, all dependencies must specify a version number, not just a path. The `path` specification is automatically stripped during packaging, and the published package will use the version from crates.io.

### Impact
- [x] Documentation only
- [ ] Breaking change
- [ ] New feature
- [ ] Bug fix

### Publishing Order
Due to workspace dependency structure:
1. Publish `kube-condition-derive` first (no runtime dependency on kube-condition)
2. Publish `kube-condition` second (depends on kube-condition-derive from crates.io)

### Added
- Initial project structure for kube-condition library
- `kube-condition` runtime crate with StatusCondition trait
- `kube-condition-derive` proc macro crate for deriving StatusCondition
- ConditionInfo and Severity types for mapping errors to conditions
- ConditionBuilder for fluent condition creation
- ConditionExt trait for updating resource status conditions
- reconcile_with_status wrapper for automatic status updates
- Support for custom condition types, reasons, and severity levels
- Configurable retry behavior per error variant
- Comprehensive documentation and examples
- `.cargo/audit.toml` to ignore unmaintained dependency warnings from kube-rs 0.96

### Changed
- **BREAKING**: Renamed `#[condition(type = "...")]` attribute to `#[condition(condition_type = "...")]` to avoid Rust keyword conflicts
- All security-scan action calls now specify `cargo-audit-version: '0.22.0'` to support CVSS 4.0 advisories
- README.md: Added comprehensive badges grouped by CI/CD Status, Code Quality, Crates.io, and License & Compliance
- README.md: Updated all examples to use `condition_type` instead of `type`

## [2025-12-21 12:30] - Fix Release Workflow for Workspace Version Management

**Author:** Erick Bourgeois

### Changed
- `.github/workflows/release.yaml`: Fixed `package-crates` and `publish-crates` jobs to work with workspace version inheritance
- Changed from `cd ${{ matrix.crate.path }} && cargo package` to `cargo package --package ${{ matrix.crate.name }}`
- Changed from `cd ${{ matrix.crate.path }} && cargo publish` to `cargo publish --package ${{ matrix.crate.name }}`

### Why
**Workspace Version Compatibility:**
- The crates use `version.workspace = true` which requires commands to be run from the workspace root
- Running `cargo package` or `cargo publish` from individual crate directories fails because the workspace version is not accessible
- Using `--package <name>` flag from the workspace root allows cargo to properly resolve workspace-inherited fields

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-21 12:15] - Add Multi-Architecture Testing to Main Workflow

**Author:** Erick Bourgeois

### Changed
- `.github/workflows/main.yaml`: Added matrix strategy to build and test jobs for both linux-x86_64 and linux-arm64 architectures
- Build job now runs on both `ubuntu-latest` (linux-x86_64) and `ubuntu-latest` (linux-arm64)
- Test job now runs on both `ubuntu-latest` (linux-x86_64) and `ubuntu-latest` (linux-arm64)
- Coverage job clarified to run only on linux-x86_64 platform

### Why
**Cross-Platform Compatibility:**
- Ensure library works correctly on both x86-64 and ARM64 architectures in main branch CI
- Maintain consistency with PR workflow multi-architecture testing
- Catch architecture-specific bugs early before merging to main
- Provide confidence for users deploying on ARM64 Kubernetes clusters (e.g., AWS Graviton, Raspberry Pi clusters)

### Impact
- [ ] Breaking change
- [x] New feature
- [ ] Bug fix
- [ ] Documentation only

## [2025-12-21 12:00] - Upgrade GitHub Actions to v1.2.4

**Author:** Erick Bourgeois

### Changed
- Upgraded all `firestoned/github-actions` references from v1.2.3 to v1.2.4 across all workflows (29 references)
- Updated workflows: pr.yaml, main.yaml, release.yaml, sbom.yml, security-scan.yaml

### Why
**Version Upgrade:**
- Incorporate latest fixes and improvements from firestoned/github-actions
- Includes the SBOM generation target directory check fix
- Maintain consistency across all workflow files

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-21 11:45] - Upgrade GitHub Actions to v1.2.3

**Author:** Erick Bourgeois

### Changed
- Upgraded all `firestoned/github-actions` references from v1.2.2 to v1.2.3 across all workflows (29 references)
- Updated workflows: pr.yaml, main.yaml, release.yaml, sbom.yml, security-scan.yaml

### Why
**Version Upgrade:**
- Incorporate latest fixes and improvements from firestoned/github-actions
- Includes the SBOM generation exit code fix (compgen vs ls)
- Maintain consistency across all workflow files

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-21 11:30] - Fix SBOM Generation Script Exit Code

**Author:** Erick Bourgeois

### Changed
- Fixed `firestoned/github-actions/rust/generate-sbom` action to use `compgen` instead of `ls` for checking SBOM file existence
- Added directory existence check before running `find target` in summary count section
- This prevents bash from exiting with code 1 when iterating through workspace directories or when target directory doesn't exist

### Why
**SBOM Generation Exit Code Fix:**
- The generate-sbom action was successfully generating SBOMs but failing with exit code 1
- **Issue 1**: When iterating through workspace Cargo.toml files, the script checked the root workspace directory. The root directory doesn't contain SBOM files (they're in subdirectories), so `ls` returned non-zero
- **Issue 2**: The summary count section runs `find target` which fails if the target directory doesn't exist (e.g., after cargo package)
- With bash running in `-e -o pipefail` mode, these failures caused the script to exit with error code 1
- **Fix 1**: Switched to `compgen -G` which properly checks file existence without causing exit code issues
- **Fix 2**: Added `if [ -d "target" ]` check before attempting to find SBOMs in target directory

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-21 11:15] - Fix SBOM Generation Workspace Configuration

**Author:** Erick Bourgeois

### Changed
- `.github/workflows/pr.yaml`: Added `workspace: true` to SBOM generation step
- `.github/workflows/main.yaml`: Added `workspace: true` to SBOM generation step
- `.github/workflows/sbom.yml`: Added `workspace: true` to SBOM generation step
- `.github/workflows/release.yaml`: Added `workspace: true` to SBOM generation step

### Why
**SBOM Generation Fix:**
- The generate-sbom action was failing because it defaulted to `workspace: false` but still used `--all` flag
- This caused cargo-cyclonedx to generate SBOMs in individual crate directories
- The verification step only checked the root directory, causing a mismatch
- Explicitly setting `workspace: true` ensures the action correctly searches all crate directories for generated SBOMs

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-21 11:00] - Add Multi-Architecture Testing to PR Workflow

**Author:** Erick Bourgeois

### Changed
- `.github/workflows/pr.yaml`: Added matrix strategy to build and test jobs for both linux-x86_64 and linux-arm64 architectures
- Build job now runs on both `ubuntu-latest` (linux-x86_64) and `ubuntu-24.04-arm64` (linux-arm64)
- Test job now runs on both `ubuntu-latest` (linux-x86_64) and `ubuntu-24.04-arm64` (linux-arm64)
- Coverage job clarified to run only on linux-x86_64 platform

### Why
**Cross-Platform Compatibility:**
- Ensure library works correctly on both x86-64 and ARM64 architectures
- Catch architecture-specific bugs early in the PR review process
- Provide confidence for users deploying on ARM64 Kubernetes clusters (e.g., AWS Graviton, Raspberry Pi clusters)

### Impact
- [ ] Breaking change
- [x] New feature
- [ ] Bug fix
- [ ] Documentation only

## [2025-12-20 10:50] - Fix Code Formatting

**Author:** Erick Bourgeois

### Changed
- `kube-condition-derive/src/lib.rs`: Applied rustfmt formatting to closure definition (lines 97-104)
- `kube-condition-derive/tests/integration.rs`: Applied rustfmt formatting to struct initialization (line 129)

### Why
**CI Compliance:**
- Fix formatting issues detected by rust/lint composite action
- Ensure code passes `cargo fmt --all --check` in CI workflows
- Maintain consistent code style across the project

### Impact
- [ ] Breaking change
- [ ] New feature
- [ ] Bug fix
- [x] Documentation only

## [2025-12-20 10:45] - Migrate to rust/lint Composite Action

**Author:** Erick Bourgeois

### Changed
- `.github/workflows/main.yaml`: Replaced manual lint steps with `firestoned/github-actions/rust/lint@v1.2.2`
  - Removed manual `rustup component add rustfmt clippy` step
  - Removed `make fmt` and `make clippy` commands
  - Now uses centralized lint action with workspace support
  - Added pedantic clippy lints with exception for module_name_repetitions
- `.github/workflows/pr.yaml`: Applied same lint action migration
  - Consistent linting approach across all workflows
  - Simplified workflow configuration

### Why
**Standardization and Maintainability:**
- Use centralized, tested lint logic from firestoned/github-actions
- Eliminate duplicate Makefile-based lint commands in workflows
- Leverage composite action features (workspace support, configurable clippy args)
- Maintain consistency across all firestoned projects
- Easier to update linting rules globally

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-20 10:30] - Upgrade to firestoned/github-actions@v1.2.2

**Author:** Erick Bourgeois

### Changed
- All GitHub Actions workflows upgraded from `firestoned/github-actions@v1.2.0` to `@v1.2.2`
  - `.github/workflows/main.yaml`: 8 action references updated
  - `.github/workflows/pr.yaml`: 8 action references updated
  - `.github/workflows/release.yaml`: 6 action references updated
  - `.github/workflows/sbom.yml`: 2 action references updated
  - `.github/workflows/security-scan.yaml`: 1 action reference updated

### Why
**Stay Current with Latest Improvements:**
- Benefit from latest bug fixes and improvements in the centralized actions repository
- Maintain consistency across all firestoned projects
- Ensure compatibility with latest composite action features

### Impact
- [ ] Breaking change
- [ ] New feature
- [ ] Bug fix
- [x] Documentation only

## [2025-12-20 10:15] - Upgrade to firestoned/github-actions@v1.2.0

**Author:** Erick Bourgeois

### Changed
- All GitHub Actions workflows upgraded from `firestoned/github-actions@v1.1.2` to `@v1.2.0`
  - `.github/workflows/main.yaml`: 8 action references updated
  - `.github/workflows/pr.yaml`: 8 action references updated
  - `.github/workflows/release.yaml`: 6 action references updated
  - `.github/workflows/sbom.yml`: 2 action references updated
  - `.github/workflows/security-scan.yaml`: 1 action reference updated

### Why
**Stay Current with Latest Improvements:**
- Benefit from latest bug fixes and improvements in the centralized actions repository
- Maintain consistency across all firestoned projects
- Ensure compatibility with latest composite action features

### Impact
- [ ] Breaking change
- [ ] New feature
- [ ] Bug fix
- [x] Documentation only

## [2025-12-20 09:30] - Migrate release.yaml to Use Reusable Composite Action

**Author:** Erick Bourgeois

### Changed
- `.github/workflows/release.yaml`: Replaced manual version extraction script with `firestoned/github-actions/versioning/extract-version@v1.1.2` composite action
  - Removed manual bash script for extracting version from tag
  - Removed incorrect Cargo.toml update step from extract-version job (was trying to reference job outputs from within same job)
  - Updated output reference from `tag_name` to `tag-name` (hyphenated) to match composite action output format
  - Simplified extract-version job to use standardized version extraction logic
  - Added workspace version update step in both `package-crates` and `publish-crates` jobs to update root `Cargo.toml`
  - Changed version update from individual crate `Cargo.toml` files to workspace root `Cargo.toml` (workspace inheritance pattern)

### Why
**Consistency and Correctness:**
- Uses the same version extraction logic across all firestoned projects
- Eliminates workflow-specific bash scripting in favor of tested composite actions
- Fixes incorrect self-referencing of job outputs (was using `needs.extract-version.outputs.version` within the extract-version job itself)
- Properly updates workspace version in root `Cargo.toml` which propagates to all crates via `version.workspace = true`
- Follows the DRY principle for GitHub Actions workflows

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix
- [ ] Documentation only

## [2025-12-19 16:45] - Enhanced README Badge Organization

**Author:** Erick Bourgeois

### Changed
- **README.md badge organization** restructured to match firestoned/github-actions format:
  - Added "Project Status" section with License, GitHub Release, Crates.io, Downloads, and Last Commit badges
  - Reorganized "CI/CD Status" section for better clarity
  - Renamed "Code Quality" to "Code Quality & Testing" with improved badge selection
  - Added "Technology & Compatibility" section with Kubernetes, Rust, kube-rs, and Proc Macro badges
  - Enhanced "Security & Compliance" section with SPDX, SBOM, Cosign, and Security Audit badges
  - Added "Community & Support" section with Issues, PRs, Contributors, and Stars badges
  - Added project tagline: "Type-safe Kubernetes status conditions for Rust operators"
  - Updated description to emphasize production-readiness and supply chain security

### Why
**Improved Project Presentation:**
- Consistent badge organization across firestoned projects
- Better categorization makes it easier to find relevant information
- Technology badges clearly communicate dependencies and compatibility
- Community badges encourage engagement and contributions
- Enhanced visibility of security and compliance features

### Impact
- [ ] Breaking change
- [ ] New feature
- [ ] Bug fix
- [x] Documentation only

## [2025-12-19 16:30] - Upgrade to firestoned/github-actions@v1.1.2

**Author:** Erick Bourgeois

### Changed
- **All workflows upgraded from v1.1.1 to v1.1.2** with updated action paths:
  - `.github/workflows/main.yaml`: All actions upgraded to v1.1.2
  - `.github/workflows/pr.yaml`: All actions upgraded to v1.1.2
  - `.github/workflows/release.yaml`: All actions upgraded to v1.1.2
  - `.github/workflows/sbom.yml`: All actions upgraded to v1.1.2
  - `.github/workflows/security-scan.yaml`: Security scan action upgraded to v1.1.2

### Why
**Version Upgrade:**
- Upgraded from v1.1.1 to v1.1.2
- Contains latest bug fixes and improvements from firestoned/github-actions
- Maintains all existing functionality with enhanced reliability

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix (v1.1.2 upgrade)
- [ ] Documentation only

## [2025-12-19 16:15] - Fix SBOM Artifact Upload Paths

**Author:** Erick Bourgeois

### Changed
- **All SBOM upload paths** now use recursive glob patterns to capture SBOM files in subdirectories:
  - `.github/workflows/sbom.yml`: Changed from `*.cdx.json` to `**/*.cdx.json` and `*.cdx.xml` to `**/*.cdx.xml`
  - `.github/workflows/main.yaml`: Changed from `*.cdx.json` to `**/*.cdx.json`
  - `.github/workflows/pr.yaml`: Changed from `*.cdx.json` to `**/*.cdx.json`
  - `.github/workflows/release.yaml`: Changed from `*.cdx.json` to `**/*.cdx.json`
  - `.github/workflows/sbom.yml`: Updated SBOM file discovery to use `find . -name "*.cdx.json"` instead of `ls *.cdx.json`

### Why
**SBOM File Location:**
- `cargo cyclonedx --all --describe crate` generates SBOM files in individual crate directories
- For workspace projects, SBOMs are created at: `kube-condition/kube-condition.cdx.json` and `kube-condition-derive/kube-condition-derive.cdx.json`
- Root-level glob patterns (`*.cdx.json`) don't capture files in subdirectories
- Recursive glob patterns (`**/*.cdx.json`) correctly find all SBOM files regardless of depth

**Error Before Fix:**
```
Error: Unable to download artifact(s): Artifact not found for name: sbom
Please ensure that your artifact is not expired and the artifact was uploaded using a compatible version of toolkit/upload-artifact.
```

**Root Cause:**
- Upload action couldn't find SBOM files because they were in subdirectories
- Artifact uploads failed silently, creating empty artifacts
- Download actions failed with "Artifact not found" error

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix (SBOM artifact uploads now work correctly)
- [ ] Documentation only

## [2025-12-19 16:00] - Upgrade to firestoned/github-actions@v1.1.1

**Author:** Erick Bourgeois

### Changed
- **All workflows upgraded from v1.1.0 to v1.1.1** with updated action paths:
  - `.github/workflows/main.yaml`: All actions upgraded to v1.1.1
  - `.github/workflows/pr.yaml`: All actions upgraded to v1.1.1
  - `.github/workflows/release.yaml`: All actions upgraded to v1.1.1
  - `.github/workflows/sbom.yml`: All actions upgraded to v1.1.1
  - `.github/workflows/security-scan.yaml`: Security scan action upgraded to v1.1.1

### Why
**Version Upgrade:**
- Upgraded from v1.1.0 to v1.1.1
- Contains latest bug fixes and improvements from firestoned/github-actions
- Maintains all existing functionality with enhanced reliability

### Impact
- [ ] Breaking change
- [ ] New feature
- [x] Bug fix (v1.1.1 upgrade)
- [ ] Documentation only

## [2025-12-19 15:30] - Fix SBOM Generation for Library Crates

**Author:** Erick Bourgeois

### Changed
- **All SBOM generation workflows** now use `describe: crate` instead of `describe: binaries`:
  - `.github/workflows/sbom.yml`: Changed from `describe: binaries` to `describe: crate`
  - `.github/workflows/main.yaml`: Added `describe: crate` parameter
  - `.github/workflows/pr.yaml`: Added `describe: crate` parameter
  - `.github/workflows/release.yaml`: Added `describe: crate` parameter

### Why
**Library Crate SBOM Requirements:**
- Library crates do not produce binary artifacts
- `cargo cyclonedx --describe binaries` produces no output for libraries (only works for bin/cdylib targets)
- `describe: crate` describes the entire crate with all Cargo targets as subcomponents (default behavior)
- This ensures SBOMs are correctly generated for both `kube-condition` and `kube-condition-derive` library crates

**cargo-cyclonedx Describe Options:**
- `crate` (default): Describe entire crate in single SBOM with targets as subcomponents
- `binaries`: Separate SBOM per binary (bin, cdylib) - ignores lib crates
- `all-cargo-targets`: Separate SBOM per Cargo target (including rlib)

**Why `crate` for Library Projects:**
- Provides comprehensive view of the entire crate and its dependencies
- Works for both library and binary crates
- Single SBOM file per crate for easier management
- Includes all targets (lib, proc-macro, etc.) as subcomponents

### Impact
- [ ] Breaking change
- [x] Bug fix (SBOM generation now works for library crates)
- [ ] New feature
- [ ] Documentation only

## [2025-12-19 14:56] - Upgrade to firestoned/github-actions@v1.1.0 with build-library Action

**Author:** Erick Bourgeois

### Changed
- **All workflows upgraded to v1.1.0** with correct action paths:
  - `.github/workflows/main.yaml`:
    - Build: `firestoned/github-actions/rust/build-library@v1.1.0` with `workspace: true`
    - All other jobs: `firestoned/github-actions/rust/setup-rust-build@v1.1.0` with `target: x86_64-unknown-linux-gnu`
    - Security scan: `firestoned/github-actions/rust/security-scan@v1.1.0`
    - SBOM generation: `firestoned/github-actions/rust/generate-sbom@v1.1.0`
    - Lint job: Added manual `rustup component add rustfmt clippy` after Rust setup
  - `.github/workflows/pr.yaml`:
    - Build: `firestoned/github-actions/rust/build-library@v1.1.0` with `workspace: true`
    - All other jobs: `firestoned/github-actions/rust/setup-rust-build@v1.1.0` with `target: x86_64-unknown-linux-gnu`
    - Security scan: `firestoned/github-actions/rust/security-scan@v1.1.0`
    - SBOM generation: `firestoned/github-actions/rust/generate-sbom@v1.1.0`
    - Lint job: Added manual `rustup component add rustfmt clippy` after Rust setup
  - `.github/workflows/release.yaml`:
    - Rust setup: `firestoned/github-actions/rust/setup-rust-build@v1.1.0` with `target: x86_64-unknown-linux-gnu`
    - Security scan: `firestoned/github-actions/rust/security-scan@v1.1.0`
    - SBOM generation: `firestoned/github-actions/rust/generate-sbom@v1.1.0`
    - Cosign signing: `firestoned/github-actions/security/cosign-sign@v1.1.0`
  - `.github/workflows/sbom.yml`:
    - Rust setup: `firestoned/github-actions/rust/setup-rust-build@v1.1.0` with `target: x86_64-unknown-linux-gnu`
    - SBOM generation: `firestoned/github-actions/rust/generate-sbom@v1.1.0`
  - `.github/workflows/security-scan.yaml`:
    - Security scan: `firestoned/github-actions/rust/security-scan@v1.1.0`

### Why
**Version Upgrade:**
- Upgraded from v1.0.0 to v1.1.0
- Introduced `rust/build-library` action for optimized library builds
- Consolidated Rust setup using `rust/setup-rust-build` reusable action

**build-library Action Benefits:**
- Specifically designed for Rust library projects (not binaries)
- Supports workspace builds with `workspace: true` parameter
- Includes internal caching for optimal build performance
- Intelligent build tool selection (cargo vs cross for ARM64)
- Flexible configuration: profiles, features, targets

**setup-rust-build Action Benefits:**
- Reusable action that combines `dtolnay/rust-toolchain` + `Swatinem/rust-cache`
- Provides consistent Rust environment setup across all jobs
- Eliminates duplicate setup logic in workflows
- Supports cross-compilation with target architecture specification
- Includes automatic dependency caching with target-specific cache keys

**Correct Action Paths:**
- Fixed action references to use proper directory structure (`rust/`, `security/`)
- Aligned with firestoned/github-actions repository organization
- Ensures actions resolve correctly at runtime

### Impact
- [ ] Breaking change
- [x] New feature (v1.1.0 upgrade + build-library action)
- [ ] Bug fix
- [ ] Documentation only

### Migration Notes
- All workflows now use `firestoned/github-actions@v1.1.0`
- Build jobs now use `rust/build-library@v1.1.0` with `workspace: true`
- All non-build jobs use `rust/setup-rust-build@v1.1.0` with `target: x86_64-unknown-linux-gnu`
- Action paths include category directories (rust/, security/, docker/, versioning/)
- Lint jobs require manual `rustup component add rustfmt clippy` after Rust setup
- Both `build-library` and `setup-rust-build` handle caching internally

## [2025-12-19 00:00] - CI/CD Workflows: PR, Main, and Release

**Author:** Erick Bourgeois

### Added
- `.github/workflows/pr.yaml`: Pull Request CI workflow
  - Lint (fmt + clippy)
  - Build all crates
  - Run all tests
  - Build documentation
  - Security audit
  - Code coverage with codecov
  - SBOM generation
- `.github/workflows/main.yaml`: Main branch CI/CD workflow
  - Lint (fmt + clippy)
  - Build all crates
  - Run all tests
  - Build documentation
  - Security audit
  - Code coverage
  - SBOM generation
- `.github/workflows/release.yaml`: Release workflow
  - Extract version from tag
  - Package both crates (kube-condition-derive and kube-condition)
  - Sign crate packages and SBOMs with Cosign (keyless)
  - Publish to crates.io (sequential: derive first, then runtime)
  - Upload signed artifacts to GitHub releases
  - Security vulnerability scan

### Changed
- None

### Why
**CI/CD Automation:**
- **Pull Request Validation**: Automated checks for code quality, tests, security, and coverage
- **Main Branch Protection**: Continuous validation of main branch with same checks as PRs
- **Release Automation**: Complete release process from packaging to signing to publishing
- **Sequential Publishing**: kube-condition-derive published first (dependency for kube-condition)
- **Artifact Signing**: All .crate files and SBOMs signed with Cosign for supply chain security

**Based on:**
- Adapted from `~/dev/bindy` workflows (binary with Docker)
- Adapted from `~/dev/bindcar` workflows (library crate publishing)
- Customized for kube-condition (proc macro + runtime library, no Docker)

### Impact
- [ ] Breaking change
- [x] New feature
- [ ] Bug fix
- [ ] Documentation only

### Security
- All release artifacts (crates and SBOMs) signed with Cosign
- Signatures recorded in Rekor transparency log
- Security audits run on PR, main, and release workflows

## [2025-12-18 23:30] - Supply Chain Security: SBOM and Signing Implementation

**Author:** Erick Bourgeois

### Added
- `.github/actions/generate-sbom/`: Composite action for SBOM generation using cargo-cyclonedx
- `.github/actions/cosign-sign/`: Composite action for keyless signing with Cosign (Sigstore)
- `.github/actions/security-scan/`: Composite action for vulnerability scanning with cargo-audit
- `.github/workflows/sbom.yml`: Scheduled workflow for daily SBOM generation and verification
- `.github/workflows/security-scan.yaml`: Scheduled workflow for daily security vulnerability scanning
- `Makefile` targets:
  - `make sbom`: Generate SBOM for all crates in CycloneDX JSON format
  - `make sbom-validate`: Validate SBOM files against CycloneDX specification
  - `make audit`: Run cargo-audit security scanning
  - `make security-check`: Complete security audit with JSON report generation
- `ROADMAP-SIGNING-SBOM.md`: Comprehensive roadmap for SBOM and signing implementation

### Changed
- `Makefile`: Added SBOM generation, validation, and security audit targets

### Why
**Regulatory Compliance & Supply Chain Security:**
- **Executive Order 14028** (US): Federal agencies must obtain SBOMs from software suppliers
- **EU Cyber Resilience Act**: Requires SBOM for software products
- **NIST SSDF**: Recommends SBOM as part of secure software development
- **Basel III / SOX Compliance**: Banking regulations require software provenance and auditability
- **Zero-trust security**: Cryptographic signing provides non-repudiation and authenticity

**Technical Benefits:**
- **Transparency**: Machine-readable dependency inventory for vulnerability management
- **Auditability**: Cosign keyless signing with Rekor transparency log creates immutable audit trail
- **Automation**: SBOM and signing integrated into CI/CD with zero manual overhead
- **No secrets management**: Keyless signing uses GitHub OIDC (no long-lived private keys)

### Impact
- [ ] Breaking change
- [x] New feature
- [ ] Bug fix
- [x] Documentation only (ROADMAP-SIGNING-SBOM.md)

### Security
- Implemented CycloneDX SBOM generation for dependency transparency
- Implemented Cosign keyless signing for artifact authenticity
- Daily vulnerability scanning with cargo-audit
- Automated GitHub issue creation for security vulnerabilities
- Compliance with FIPS 140-2 cryptographic standards (via Sigstore/Cosign)

### Changed
- None

### Deprecated
- None

### Removed
- None

### Fixed
- None

### Security
- None

## [0.1.0] - TBD

Initial release (not yet published)

---

## Changelog Entry Template

When making changes, use this template:

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
