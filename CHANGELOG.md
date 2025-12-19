# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
