# kube-condition

<!-- CI/CD Status -->
[![Main Branch CI](https://github.com/firestoned/kube-condition/actions/workflows/main.yaml/badge.svg)](https://github.com/firestoned/kube-condition/actions/workflows/main.yaml)
[![Pull Request CI](https://github.com/firestoned/kube-condition/actions/workflows/pr.yaml/badge.svg)](https://github.com/firestoned/kube-condition/actions/workflows/pr.yaml)
[![Security Scan](https://github.com/firestoned/kube-condition/actions/workflows/security-scan.yaml/badge.svg)](https://github.com/firestoned/kube-condition/actions/workflows/security-scan.yaml)
[![SBOM Generation](https://github.com/firestoned/kube-condition/actions/workflows/sbom.yml/badge.svg)](https://github.com/firestoned/kube-condition/actions/workflows/sbom.yml)

<!-- Code Quality -->
[![codecov](https://codecov.io/gh/firestoned/kube-condition/branch/main/graph/badge.svg)](https://codecov.io/gh/firestoned/kube-condition)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

<!-- Crates.io -->
[![Crates.io](https://img.shields.io/crates/v/kube-condition.svg)](https://crates.io/crates/kube-condition)
[![Documentation](https://docs.rs/kube-condition/badge.svg)](https://docs.rs/kube-condition)
[![Downloads](https://img.shields.io/crates/d/kube-condition.svg)](https://crates.io/crates/kube-condition)

<!-- License & Compliance -->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![SBOM](https://img.shields.io/badge/SBOM-CycloneDX-blue)](https://github.com/firestoned/kube-condition/releases/latest)
[![Signed Releases](https://img.shields.io/badge/Releases-Cosign%20Signed-green)](https://github.com/firestoned/kube-condition/releases/latest)

A complete example of using a custom derive macro to map Rust errors to Kubernetes status conditions in the bindy operator.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Workspace                               │
├─────────────────────┬─────────────────────┬─────────────────────┤
│ kube-condition-     │   kube-condition    │       bindy         │
│ derive              │   (runtime)         │   (operator)        │
├─────────────────────┼─────────────────────┼─────────────────────┤
│ #[derive(Status     │ StatusCondition     │ ZoneReconcileError  │
│   Condition)]       │   trait             │ RecordReconcileError│
│                     │                     │                     │
│ Generates:          │ Provides:           │ Uses:               │
│ - to_condition_info │ - ConditionExt      │ - #[condition(...)] │
│ - severity()        │ - ReadyCondition    │ - reconcile_with_   │
│ - is_retryable()    │ - ConditionBuilder  │   status()          │
│ - requeue_duration()│ - reconcile_with_   │                     │
│                     │   status()          │                     │
└─────────────────────┴─────────────────────┴─────────────────────┘
```

## Usage

### Define your errors with condition attributes

```rust
use kube_condition::StatusCondition;
use thiserror::Error;

#[derive(Error, Debug, StatusCondition)]
#[condition(default_type = "Ready")]
pub enum ZoneReconcileError {
    #[error("RNDC command '{command}' failed: {message}")]
    #[condition(
        reason = "RndcCommandFailed",
        severity = "error",
        retryable = true,
        requeue_secs = 15
    )]
    RndcCommand { command: String, message: String },

    #[error("Zone file syntax error: {details}")]
    #[condition(reason = "ZoneSyntaxError", retryable = false)]
    ZoneSyntax { zone: String, details: String },

    #[error("DNSSEC signing failed: {0}")]
    #[condition(
        condition_type = "DnssecReady",  // Custom condition type
        reason = "SigningFailed",
        retryable = true
    )]
    DnssecSigning(String),
}
```

### Attribute options

| Attribute         | Default         | Description                                    |
|-------------------|-----------------|------------------------------------------------|
| `condition_type`  | `"Ready"`       | Condition type (can override with default_type)|
| `reason`          | Variant name    | Reason string for the condition                |
| `status`          | `"False"`       | Status value (True/False/Unknown)              |
| `severity`        | `"error"`       | Logging severity (info/warning/error)          |
| `retryable`       | `true`          | Whether to retry reconciliation                |
| `requeue_secs`    | `30`            | Seconds to wait before retry                   |

### Use in reconcile loop

```rust
async fn reconcile(
    obj: Arc<DnsZone>,
    ctx: Arc<Context>,
) -> Result<Action, ZoneReconcileError> {
let generation = obj.meta().generation;

match reconcile_inner(obj.clone(), ctx.clone()).await {
Ok(action) => {
// Update status to ready
obj.set_ready(&ctx.client, "Reconciliation succeeded").await?;
Ok(action)
}
Err(e) => {
// Automatically updates status with correct condition
obj.set_error(&ctx.client, &e).await?;

// Uses error's retry policy
Ok(e.to_action())
}
}
}
```

### Or use the wrapper

```rust
use kube_condition::reconcile_with_status;

// Wrap your reconcile function - status updates happen automatically
let wrapped = reconcile_with_status!(reconcile_inner);

Controller::new(zones, Config::default())
    .run(wrapped, error_policy, ctx)
    .await;
```

## Generated trait implementation

The derive macro generates:

```rust
impl StatusCondition for ZoneReconcileError {
    fn to_condition_info(&self) -> ConditionInfo {
        match self {
            Self::RndcCommand { .. } => ConditionInfo {
                type_: "Ready".to_string(),
                status: "False".to_string(),
                reason: "RndcCommandFailed".to_string(),
                message: self.to_string(),
            },
            // ... other variants
        }
    }

    fn severity(&self) -> Severity {
        match self {
            Self::RndcCommand { .. } => Severity::Error,
            // ...
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::RndcCommand { .. } => true,
            Self::ZoneSyntax { .. } => false,
            // ...
        }
    }

    fn requeue_duration(&self) -> Duration {
        match self {
            Self::RndcCommand { .. } => Duration::from_secs(15),
            // ...
        }
    }
}
```

## Benefits

1. **Single source of truth** - Error behavior defined alongside error types
2. **Type safety** - Compile-time guarantees for condition mappings
3. **Consistency** - All errors handled uniformly
4. **Observability** - Severity levels for metrics/alerting
5. **Retry control** - Fine-grained retry policies per error type

## Files

```
bindy-status-example/
├── Cargo.toml                    # Workspace definition
├── kube-condition-derive/
│   ├── Cargo.toml
│   └── src/lib.rs               # Proc macro implementation
├── kube-condition/
│   ├── Cargo.toml
│   └── src/lib.rs               # Runtime traits and helpers
└── bindy/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── main.rs
        ├── crd.rs               # DnsZone, DnsRecord CRDs
        ├── error.rs             # Error types with derive macro
        ├── controller.rs        # Reconcile loop
        └── rndc.rs              # RNDC client (placeholder)
```

## Security

### Software Bill of Materials (SBOM)

SBOM files in CycloneDX format are generated for each release and attached to GitHub releases:
- **kube-condition SBOM**: Download from [latest release](https://github.com/YOUR_ORG/kube-condition/releases/latest)
- **kube-condition-derive SBOM**: Download from [latest release](https://github.com/YOUR_ORG/kube-condition/releases/latest)

Generate SBOM locally:
```bash
make sbom
```

### Signed Releases

All release artifacts are cryptographically signed using [Sigstore Cosign](https://docs.sigstore.dev/) with keyless signing.

**Verify a release:**

```bash
# Install Cosign
brew install cosign  # macOS
# OR download from: https://github.com/sigstore/cosign/releases

# Download SBOM and signature
curl -sL https://github.com/YOUR_ORG/kube-condition/releases/latest/download/kube-condition.cdx.json -o kube-condition.cdx.json
curl -sL https://github.com/YOUR_ORG/kube-condition/releases/latest/download/kube-condition.cdx.json.bundle -o kube-condition.cdx.json.bundle

# Verify signature
cosign verify-blob \
  --bundle kube-condition.cdx.json.bundle \
  --certificate-identity-regexp "^https://github.com/YOUR_ORG/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition.cdx.json
```

**Expected output:** `Verified OK`

See [Verifying Artifacts](docs/security/verifying-artifacts.md) for detailed instructions.

### Security Scanning

- **Daily vulnerability scanning** with `cargo-audit`
- **Automated dependency updates** via Dependabot
- **SBOM vulnerability checks** with Grype

Run security scan locally:
```bash
make audit
```

### Compliance

This project implements supply chain security best practices:
- ✅ **SBOM Generation**: CycloneDX format, compliant with EO 14028 and EU Cyber Resilience Act
- ✅ **Artifact Signing**: Keyless signing with Sigstore/Cosign (FIPS 140-2 compliant)
- ✅ **Transparency Log**: All signatures recorded in Rekor for auditability
- ✅ **Vulnerability Scanning**: Daily automated scans with cargo-audit
- ✅ **SLSA Provenance**: Build provenance attestation (planned)

For regulated environments (banking, healthcare, government):
- Meets **NIST SSDF** (Secure Software Development Framework) requirements
- Supports **SOX** audit trails with immutable signature logs
- Complies with **Basel III** operational risk management for cyber resilience

## Future enhancements

- [ ] Support for multiple conditions per error
- [ ] Prometheus metrics integration
- [ ] Condition history tracking
- [ ] Event generation alongside status updates
- [ ] SLSA Level 3 provenance attestation
