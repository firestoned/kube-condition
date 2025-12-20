# Roadmap: Signing & SBOM for Rust Crates

> **Project:** kube-condition
> **Goal:** Implement cryptographic signing and SBOM generation for crates.io publishing
> **Target:** Enhanced supply chain security and compliance
> **Author:** Erick Bourgeois
> **Created:** 2025-12-18

---

## Overview

This roadmap covers two critical supply chain security practices:

1. **SBOM (Software Bill of Materials)**: Machine-readable inventory of dependencies
2. **Cryptographic Signing**: Verify authenticity and integrity of releases

**Why This Matters:**
- **Regulatory Compliance**: Executive Order 14028 (US) and EU Cyber Resilience Act require SBOMs
- **Supply Chain Security**: Verify artifact authenticity and detect tampering
- **Transparency**: Enable dependency tracking and vulnerability management
- **Trust**: Build confidence in your published crates

---

## Phase 1: SBOM Generation (Start Here - Easier)

### 1.1 Choose SBOM Format & Tooling

**Decision Point:** Select SBOM format(s)

**Options:**
- **SPDX** (Software Package Data Exchange) - ISO/IEC standard, widely adopted
- **CycloneDX** - OWASP project, security-focused, excellent Rust support
- **Both** - Maximum compatibility (recommended for regulated environments)

**Recommended Tooling:**

| Tool | Format | Integration | Maturity | Notes |
|------|--------|-------------|----------|-------|
| `cargo-sbom` | SPDX, CycloneDX | CLI | Stable | **Recommended** - Best Rust support |
| `cargo-cyclonedx` | CycloneDX | CLI | Stable | Specialized for CycloneDX |
| `syft` | SPDX, CycloneDX | CLI (multi-lang) | Mature | Anchore OSS tool, language-agnostic |

**Recommendation:** Use `cargo-sbom` with CycloneDX JSON format for security-focused projects.

**Action Items:**
- [ ] Install SBOM generation tool
- [ ] Test SBOM generation locally
- [ ] Choose output format(s)
- [ ] Document SBOM format decision

### 1.2 Local SBOM Generation

**Install `cargo-sbom`:**

```bash
# Install
cargo install cargo-sbom

# Verify installation
cargo sbom --version
```

**Generate SBOM files:**

```bash
# Generate SBOM for a crate (CycloneDX JSON)
cargo sbom --output-format cyclonedx_json > kube-condition.cdx.json

# Generate SBOM (SPDX JSON)
cargo sbom --output-format spdx_json_2_3 > kube-condition.spdx.json

# For all workspace crates
for crate in kube-condition kube-condition-derive; do
    echo "Generating SBOM for $crate..."
    cargo sbom --manifest-path $crate/Cargo.toml \
        --output-format cyclonedx_json > $crate.cdx.json
done
```

**SBOM File Structure:**

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.4",
  "version": 1,
  "metadata": {
    "component": {
      "type": "library",
      "name": "kube-condition",
      "version": "0.1.0"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "kube",
      "version": "0.96.0",
      "purl": "pkg:cargo/kube@0.96.0"
    }
  ]
}
```

**Action Items:**
- [ ] Generate SBOM for `kube-condition` crate
- [ ] Generate SBOM for `kube-condition-derive` crate
- [ ] Review SBOM contents - verify dependency accuracy
- [ ] Add SBOM files to `.gitignore` (generated files, not committed)
- [ ] Add SBOM generation to Makefile

**Makefile Target:**

```makefile
## Generate SBOM for all crates (CycloneDX JSON)
sbom:
	@echo "Generating SBOM for kube-condition..."
	cargo sbom --manifest-path kube-condition/Cargo.toml \
		--output-format cyclonedx_json > kube-condition.cdx.json
	@echo "Generating SBOM for kube-condition-derive..."
	cargo sbom --manifest-path kube-condition-derive/Cargo.toml \
		--output-format cyclonedx_json > kube-condition-derive.cdx.json
	@echo "✓ SBOM files generated: *.cdx.json"

## Generate SBOM in both formats
sbom-all:
	@echo "Generating SBOM in all formats..."
	$(MAKE) sbom
	cargo sbom --manifest-path kube-condition/Cargo.toml \
		--output-format spdx_json_2_3 > kube-condition.spdx.json
	cargo sbom --manifest-path kube-condition-derive/Cargo.toml \
		--output-format spdx_json_2_3 > kube-condition-derive.spdx.json
	@echo "✓ SBOM files generated in CycloneDX and SPDX formats"
```

**Update `.gitignore`:**

```gitignore
# SBOM files (generated, not committed)
*.cdx.json
*.spdx.json
*.sbom
```

### 1.3 CI/CD Integration - GitHub Actions

**Create workflow: `.github/workflows/sbom.yml`**

```yaml
name: SBOM Generation

on:
  push:
    tags:
      - 'v*'
  release:
    types: [published]
  workflow_dispatch:

permissions:
  contents: write

jobs:
  generate-sbom:
    name: Generate and Publish SBOM
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install cargo-sbom
        run: cargo install cargo-sbom

      - name: Generate SBOM files
        run: make sbom

      - name: Upload SBOM artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sbom-files
          path: '*.cdx.json'
          retention-days: 90

      - name: Attach SBOM to GitHub release
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v2
        with:
          files: |
            kube-condition.cdx.json
            kube-condition-derive.cdx.json
```

**Integrate with existing CI workflow:**

Add to `.github/workflows/ci.yml`:

```yaml
  sbom-check:
    name: Verify SBOM Generation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install cargo-sbom
        run: cargo install cargo-sbom
      - name: Verify SBOM can be generated
        run: make sbom
```

**Action Items:**
- [ ] Create `.github/workflows/sbom.yml`
- [ ] Add `make sbom` target to Makefile
- [ ] Test workflow on a test tag
- [ ] Verify SBOM files attached to GitHub releases
- [ ] Document SBOM availability in README

### 1.4 SBOM Validation & Quality

**Install validation tools:**

```bash
# Install CycloneDX CLI (requires Node.js)
npm install -g @cyclonedx/cyclonedx-cli

# OR use Docker
docker pull cyclonedx/cyclonedx-cli
```

**Validate SBOM files:**

```bash
# Validate CycloneDX SBOM
cyclonedx validate --input-file kube-condition.cdx.json

# Validate with Docker
docker run --rm -v $(pwd):/sbom cyclonedx/cyclonedx-cli \
    validate --input-file /sbom/kube-condition.cdx.json
```

**Add validation to Makefile:**

```makefile
## Validate SBOM files
sbom-validate:
	@echo "Validating SBOM files..."
	cyclonedx validate --input-file kube-condition.cdx.json
	cyclonedx validate --input-file kube-condition-derive.cdx.json
	@echo "✓ All SBOM files are valid"

## Check SBOM for known vulnerabilities
sbom-vuln-check:
	@echo "Scanning SBOM for vulnerabilities..."
	cargo install osv-scanner || true
	osv-scanner --sbom kube-condition.cdx.json || true
	osv-scanner --sbom kube-condition-derive.cdx.json || true
```

**Add to GitHub Actions:**

```yaml
      - name: Validate SBOM files
        run: |
          npm install -g @cyclonedx/cyclonedx-cli
          make sbom-validate

      - name: Scan for vulnerabilities
        run: make sbom-vuln-check
        continue-on-error: true
```

**Action Items:**
- [ ] Install `cyclonedx-cli` for validation
- [ ] Add SBOM validation to CI workflow
- [ ] Set up vulnerability scanning with `osv-scanner`
- [ ] Configure alerts for vulnerable dependencies

### 1.5 SBOM Distribution

**Distribution Channels:**

1. **GitHub Releases** (primary) - Attach SBOM to release assets
2. **crates.io metadata** - Currently no direct support (track [rust-lang/cargo#10389](https://github.com/rust-lang/cargo/issues/10389))
3. **Documentation site** - Host SBOMs alongside API docs
4. **Dependency-Track** - Upload to enterprise SBOM repository (if applicable)

**Update README.md:**

```markdown
## Security

### Software Bill of Materials (SBOM)

SBOM files in CycloneDX format are attached to each release:
- [kube-condition SBOM](https://github.com/firestoned/kube-condition/releases/latest/download/kube-condition.cdx.json)
- [kube-condition-derive SBOM](https://github.com/firestoned/kube-condition/releases/latest/download/kube-condition-derive.cdx.json)

### Verifying Dependencies

Download the SBOM and use tools like `osv-scanner` to check for vulnerabilities:

\`\`\`bash
# Download SBOM
curl -sL https://github.com/firestoned/kube-condition/releases/latest/download/kube-condition.cdx.json -o kube-condition.cdx.json

# Scan for vulnerabilities
osv-scanner --sbom kube-condition.cdx.json
\`\`\`
```

**Action Items:**
- [ ] Update README.md to mention SBOM availability
- [ ] Add SBOM section to documentation
- [ ] Test SBOM download links in README
- [ ] Create documentation page on supply chain security

---

## Phase 2: Cryptographic Signing

### 2.1 Choose Signing Method

**Options:**

| Method | Tool | Pros | Cons | Recommendation |
|--------|------|------|------|----------------|
| **Sigstore/Cosign** | `cosign` | Keyless signing, modern, transparency log | Requires Sigstore infrastructure | ✅ **Recommended** |
| **GPG Signing** | `gpg` | Traditional, widely supported | Complex key management | Use for Git commits |
| **Minisign** | `minisign` | Simple, secure | Less widely adopted | Niche use cases |

**Recommendation for 2025:** **Sigstore/Cosign** (keyless signing with OIDC)

**Why Cosign:**
- ✅ No long-lived private keys to manage
- ✅ Uses GitHub OIDC for identity verification
- ✅ Transparency log (Rekor) for auditability
- ✅ Industry standard for container signing (extends to artifacts)
- ✅ Non-repudiation via transparency log
- ✅ Automatic key rotation (no key management burden)

**Decision Rationale:**

| Feature | Cosign (Keyless) | GPG | Minisign |
|---------|------------------|-----|----------|
| Key Management | ✅ None (OIDC) | ❌ Manual | ⚠️ Simple but manual |
| Transparency Log | ✅ Rekor | ❌ No | ❌ No |
| CI/CD Integration | ✅ Excellent | ⚠️ Requires secrets | ⚠️ Requires secrets |
| Verifiability | ✅ Easy | ⚠️ Moderate | ⚠️ Moderate |
| Adoption | ✅ Growing | ✅ Widespread | ⚠️ Niche |

### 2.2 Setup Cosign Signing (Local Testing)

**Install Cosign:**

```bash
# macOS
brew install cosign

# Linux (binary download)
COSIGN_VERSION=$(curl -s https://api.github.com/repos/sigstore/cosign/releases/latest | grep tag_name | cut -d '"' -f 4)
wget "https://github.com/sigstore/cosign/releases/download/${COSIGN_VERSION}/cosign-linux-amd64"
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign

# Verify installation
cosign version
```

**Local Testing (with test keypair):**

```bash
# Generate test keypair (for local testing ONLY - not for production)
cosign generate-key-pair

# This creates:
# - cosign.key (private key, password-protected)
# - cosign.pub (public key)

# Sign a file
cosign sign-blob --key cosign.key kube-condition.cdx.json > kube-condition.cdx.json.sig

# Verify signature
cosign verify-blob \
    --key cosign.pub \
    --signature kube-condition.cdx.json.sig \
    kube-condition.cdx.json

# Expected output: "Verified OK"
```

**⚠️ IMPORTANT:** The keypair method is for **LOCAL TESTING ONLY**. In production, use keyless signing with GitHub OIDC.

**Action Items:**
- [ ] Install Cosign locally
- [ ] Test signing workflow with test keypair
- [ ] Verify signature verification works
- [ ] Document verification process
- [ ] Delete test keypair (do not commit to git)

### 2.3 CI/CD Keyless Signing with GitHub OIDC

**Why Keyless Signing:**
- No secrets to manage or rotate
- Identity verified via GitHub OIDC token
- Signature recorded in Rekor transparency log
- Cryptographically binds signature to GitHub identity

**How it Works:**
1. GitHub Actions generates OIDC token for the workflow
2. Cosign requests signing certificate from Fulcio (Sigstore CA)
3. Fulcio issues short-lived certificate bound to GitHub identity
4. Artifact is signed with ephemeral key
5. Signature and certificate uploaded to Rekor transparency log

**Update `.github/workflows/sbom.yml` to include signing:**

```yaml
name: SBOM Generation & Signing

on:
  push:
    tags:
      - 'v*'
  release:
    types: [published]
  workflow_dispatch:

permissions:
  contents: write
  id-token: write  # ✅ Required for keyless signing with OIDC

jobs:
  generate-and-sign-sbom:
    name: Generate, Sign, and Publish SBOM
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install cargo-sbom
        run: cargo install cargo-sbom

      - name: Generate SBOM
        run: make sbom

      - name: Install Cosign
        uses: sigstore/cosign-installer@v3

      - name: Sign SBOM files (keyless)
        run: |
          echo "Signing kube-condition.cdx.json..."
          cosign sign-blob --yes \
            --bundle kube-condition.cdx.json.bundle \
            kube-condition.cdx.json

          echo "Signing kube-condition-derive.cdx.json..."
          cosign sign-blob --yes \
            --bundle kube-condition-derive.cdx.json.bundle \
            kube-condition-derive.cdx.json

      - name: Upload signed SBOM artifacts
        uses: actions/upload-artifact@v4
        with:
          name: signed-sbom-files
          path: |
            *.cdx.json
            *.cdx.json.bundle
          retention-days: 90

      - name: Attach to GitHub release
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v2
        with:
          files: |
            kube-condition.cdx.json
            kube-condition.cdx.json.bundle
            kube-condition-derive.cdx.json
            kube-condition-derive.cdx.json.bundle
```

**Understanding the Signature Bundle:**

The `.bundle` file contains:
- The signature itself
- The signing certificate (from Fulcio)
- Rekor transparency log entry
- All metadata needed for verification

This is a **self-contained verification artifact** - no external key needed!

**Action Items:**
- [ ] Add `id-token: write` permission to workflow
- [ ] Update workflow with keyless signing
- [ ] Test on a release tag
- [ ] Verify signature bundles are created
- [ ] Verify signatures appear in Rekor log

### 2.4 Sign Crate Archives (.crate files)

**Current State of crates.io Signing:**

As of 2025-12-18:
- ❌ crates.io does **NOT** natively support signature verification
- ⚠️ Tracking issue: [rust-lang/rfcs#3579](https://github.com/rust-lang/rfcs/pull/3579)
- ✅ You can still sign `.crate` files for distribution via other channels

**Why Sign .crate Files Anyway:**

1. **Future-proofing**: When crates.io adds support, you're ready
2. **Alternative distribution**: Users can verify downloads from GitHub releases
3. **Audit trail**: Prove integrity of your published crates
4. **Compliance**: Meet regulatory requirements for signed software

**Workflow for signing `.crate` files:**

Add to `.github/workflows/release.yml` (or create new workflow):

```yaml
name: Sign Release Artifacts

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write

jobs:
  sign-crates:
    name: Package and Sign Crates
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Package crates
        run: |
          cargo package --manifest-path kube-condition/Cargo.toml
          cargo package --manifest-path kube-condition-derive/Cargo.toml

      - name: Install Cosign
        uses: sigstore/cosign-installer@v3

      - name: Sign .crate files (keyless)
        run: |
          # Find the generated .crate files
          CRATE_FILE=$(ls target/package/kube-condition-*.crate | head -n1)
          DERIVE_CRATE_FILE=$(ls target/package/kube-condition-derive-*.crate | head -n1)

          echo "Signing $CRATE_FILE..."
          cosign sign-blob --yes \
            --bundle ${CRATE_FILE}.bundle \
            $CRATE_FILE

          echo "Signing $DERIVE_CRATE_FILE..."
          cosign sign-blob --yes \
            --bundle ${DERIVE_CRATE_FILE}.bundle \
            $DERIVE_CRATE_FILE

      - name: Upload signed artifacts
        uses: actions/upload-artifact@v4
        with:
          name: signed-crate-files
          path: |
            target/package/*.crate
            target/package/*.crate.bundle
          retention-days: 90

      - name: Attach signed crates to release
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v2
        with:
          files: |
            target/package/*.crate
            target/package/*.crate.bundle
```

**Action Items:**
- [ ] Create or update release workflow
- [ ] Test packaging and signing `.crate` files
- [ ] Verify signed crates attached to releases
- [ ] Document verification instructions for users

### 2.5 Verification Documentation

**Create `docs/security/verifying-artifacts.md`:**

```markdown
# Verifying Signed Artifacts

This guide explains how to verify the authenticity and integrity of kube-condition releases using Cosign.

## Prerequisites

Install Cosign:

\`\`\`bash
# macOS
brew install cosign

# Linux
COSIGN_VERSION=$(curl -s https://api.github.com/repos/sigstore/cosign/releases/latest | grep tag_name | cut -d '"' -f 4)
wget "https://github.com/sigstore/cosign/releases/download/${COSIGN_VERSION}/cosign-linux-amd64"
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
\`\`\`

## What Gets Signed

We sign all release artifacts:
- SBOM files (`.cdx.json`)
- Crate archives (`.crate`)

Each signed artifact has a corresponding `.bundle` file containing the signature and verification metadata.

## Verify SBOM Signature

Download the SBOM and signature bundle from the [Releases page](https://github.com/firestoned/kube-condition/releases).

\`\`\`bash
# Download SBOM and bundle
curl -sL https://github.com/firestoned/kube-condition/releases/download/v0.1.0/kube-condition.cdx.json -o kube-condition.cdx.json
curl -sL https://github.com/firestoned/kube-condition/releases/download/v0.1.0/kube-condition.cdx.json.bundle -o kube-condition.cdx.json.bundle

# Verify using keyless signing (GitHub OIDC)
cosign verify-blob \
  --bundle kube-condition.cdx.json.bundle \
  --certificate-identity-regexp "^https://github.com/firestoned/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition.cdx.json
\`\`\`

**Expected output:**
\`\`\`
Verified OK
\`\`\`

**What This Verifies:**
- ✅ The SBOM was signed by a GitHub Actions workflow
- ✅ The workflow ran in the `firestoned/kube-condition` repository
- ✅ The signature was recorded in the Rekor transparency log
- ✅ The file has not been tampered with since signing

## Verify .crate File Signature

\`\`\`bash
# Download .crate file and bundle
curl -sL https://github.com/firestoned/kube-condition/releases/download/v0.1.0/kube-condition-0.1.0.crate -o kube-condition-0.1.0.crate
curl -sL https://github.com/firestoned/kube-condition/releases/download/v0.1.0/kube-condition-0.1.0.crate.bundle -o kube-condition-0.1.0.crate.bundle

# Verify signature
cosign verify-blob \
  --bundle kube-condition-0.1.0.crate.bundle \
  --certificate-identity-regexp "^https://github.com/firestoned/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition-0.1.0.crate
\`\`\`

## Understanding the Certificate Identity

When we use keyless signing, the signature is bound to a **certificate identity**:

\`\`\`
--certificate-identity-regexp "^https://github.com/firestoned/kube-condition"
\`\`\`

This means:
- Only workflows from this repository can produce valid signatures
- Attackers cannot forge signatures (even with access to the artifact)
- The signature is cryptographically bound to the GitHub repository

## Transparency Log (Rekor)

All signatures are recorded in the public Rekor transparency log.

You can search for signatures:

\`\`\`bash
# Search Rekor log for signatures from our repository
rekor-cli search --email noreply@github.com
\`\`\`

This provides:
- **Tamper evidence**: Signatures cannot be backdated or removed
- **Auditability**: Anyone can verify when artifacts were signed
- **Non-repudiation**: Proof that the artifact came from our CI/CD pipeline

## Troubleshooting

### "signature verification failed"

Possible causes:
1. The file was modified after signing
2. The signature bundle doesn't match the file
3. The certificate identity doesn't match

### "certificate identity does not match"

Ensure you're using the correct `--certificate-identity-regexp` for the repository.

### "connection to rekor.sigstore.dev failed"

Check your internet connection. Verification requires access to the Rekor transparency log.

## Security Considerations

- **Do not trust unsigned artifacts**: Always verify signatures before using release artifacts
- **Verify the certificate identity**: Ensure it matches `firestoned/kube-condition`
- **Check the transparency log**: Verify the signature appears in Rekor
- **Use the latest Cosign**: Keep Cosign updated for security fixes

## Questions?

If you have questions about artifact verification, please open an issue on GitHub.
```

**Update README.md:**

```markdown
## Security

### Signed Releases

All release artifacts are cryptographically signed using [Sigstore Cosign](https://docs.sigstore.dev/).

**Verify a release:**

\`\`\`bash
# Install Cosign
brew install cosign  # macOS

# Download SBOM and signature
curl -sL https://github.com/firestoned/kube-condition/releases/latest/download/kube-condition.cdx.json -o kube-condition.cdx.json
curl -sL https://github.com/firestoned/kube-condition/releases/latest/download/kube-condition.cdx.json.bundle -o kube-condition.cdx.json.bundle

# Verify signature
cosign verify-blob \
  --bundle kube-condition.cdx.json.bundle \
  --certificate-identity-regexp "^https://github.com/firestoned/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition.cdx.json
\`\`\`

See [Verifying Artifacts](docs/security/verifying-artifacts.md) for detailed instructions.
```

**Action Items:**
- [ ] Create `docs/security/` directory
- [ ] Write `verifying-artifacts.md` documentation
- [ ] Add verification section to README.md
- [ ] Test verification steps as an end user
- [ ] Create quick-start verification script

---

## Phase 3: Advanced Supply Chain Security

### 3.1 Provenance Attestation (SLSA)

**What is SLSA?**

SLSA (Supply-chain Levels for Software Artifacts) is a framework for ensuring artifact integrity.

**SLSA Levels:**
- **SLSA 1**: Documentation of build process
- **SLSA 2**: Tamper-resistant build service
- **SLSA 3**: Provenance attestation + non-falsifiable metadata
- **SLSA 4**: Two-party review + hermetic builds

**GitHub Actions SLSA Support:**

GitHub provides official SLSA attestation generation:

```yaml
name: SLSA Provenance

on:
  push:
    tags:
      - 'v*'

permissions:
  id-token: write
  contents: write
  attestations: write

jobs:
  provenance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Package crates
        run: |
          cargo package --manifest-path kube-condition/Cargo.toml
          cargo package --manifest-path kube-condition-derive/Cargo.toml

      - name: Generate SLSA provenance attestation
        uses: actions/attest-build-provenance@v1
        with:
          subject-path: |
            target/package/kube-condition-*.crate
            target/package/kube-condition-derive-*.crate
```

**What SLSA Provides:**

- **Build provenance**: Cryptographic proof of how the artifact was built
- **Build parameters**: Exact commit, workflow, runner, etc.
- **Tamper evidence**: Detect if artifacts were modified after build
- **Auditability**: Complete build history in transparency log

**Verification:**

```bash
# Verify SLSA provenance (requires gh CLI)
gh attestation verify kube-condition-0.1.0.crate \
  --owner firestoned \
  --repo kube-condition
```

**Action Items:**
- [ ] Research SLSA for Rust ecosystem
- [ ] Implement SLSA provenance generation
- [ ] Test provenance verification
- [ ] Document SLSA provenance in security docs
- [ ] Aim for SLSA Level 3 compliance

### 3.2 Dependency Scanning & Vulnerability Reporting

**Integrate `cargo-audit` and `osv-scanner`:**

Create `.github/workflows/security-scan.yml`:

```yaml
name: Security Scan

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday at midnight
  push:
    branches: [main, master]
  pull_request:
  workflow_dispatch:

permissions:
  contents: read
  security-events: write

jobs:
  scan-dependencies:
    name: Scan Dependencies for Vulnerabilities
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run cargo-audit
        run: |
          cargo install cargo-audit
          cargo audit --deny warnings

      - name: Generate SBOM
        run: |
          cargo install cargo-sbom
          make sbom

      - name: Scan SBOM with osv-scanner
        run: |
          cargo install osv-scanner
          osv-scanner --sbom kube-condition.cdx.json
          osv-scanner --sbom kube-condition-derive.cdx.json

  scan-code:
    name: Static Analysis with Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run clippy (security checks)
        run: |
          cargo clippy --all -- \
            -W clippy::all \
            -W clippy::pedantic \
            -W clippy::nursery \
            -W clippy::cargo
```

**Add to Makefile:**

```makefile
## Run security audit on dependencies
audit:
	cargo audit

## Scan SBOM for vulnerabilities
vuln-scan:
	osv-scanner --sbom kube-condition.cdx.json || true
	osv-scanner --sbom kube-condition-derive.cdx.json || true

## Complete security check
security-check: audit vuln-scan
	@echo "✓ Security checks complete"
```

**Action Items:**
- [ ] Create security scanning workflow
- [ ] Set up automated alerts (Dependabot, GitHub Security Advisories)
- [ ] Configure Dependabot for Rust dependencies
- [ ] Document vulnerability response process
- [ ] Create SECURITY.md for responsible disclosure

### 3.3 Publish to Transparency Logs

**Sigstore Rekor** automatically logs signatures when using keyless signing.

**Verify signatures appear in Rekor:**

```bash
# Install rekor-cli
go install github.com/sigstore/rekor/cmd/rekor-cli@latest

# Search for signatures from your repository
rekor-cli search --email noreply@github.com

# Get details of a specific signature
rekor-cli get --uuid <UUID>
```

**What Rekor Provides:**

- **Tamper-evident log**: Signatures cannot be removed or backdated
- **Public auditability**: Anyone can verify signature history
- **Non-repudiation**: Proof that signatures were created by your CI/CD
- **Forensics**: Investigate signing history in case of compromise

**Monitoring Rekor:**

Create a script to monitor your project's signatures:

```bash
#!/bin/bash
# monitor-rekor.sh - Monitor Rekor log for kube-condition signatures

REPO="firestoned/kube-condition"

echo "Searching Rekor log for signatures from $REPO..."

rekor-cli search \
  --pki-format x509 \
  --email noreply@github.com \
  | while read uuid; do
      echo "Found signature: $uuid"
      rekor-cli get --uuid $uuid --format json | jq '.Body.HashedRekordObj.signature.publicKey.content' -r | base64 -d | openssl x509 -text | grep "$REPO" && echo "✓ Valid signature from $REPO"
  done
```

**Action Items:**
- [ ] Verify signatures appear in Rekor log
- [ ] Document how to query Rekor for project signatures
- [ ] Create monitoring script for Rekor
- [ ] Set up alerts for unexpected signatures

### 3.4 Security Policy & Vulnerability Disclosure

**Create `SECURITY.md`:**

```markdown
# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**DO NOT** open public issues for security vulnerabilities.

Instead, please report security vulnerabilities via:

1. **GitHub Security Advisories**: [Report a vulnerability](https://github.com/firestoned/kube-condition/security/advisories/new)
2. **Email**: security@firestoned.com (encrypted email preferred)

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (if any)

### Response Timeline

- **Initial response**: Within 48 hours
- **Triage**: Within 7 days
- **Fix timeline**: Depends on severity
  - Critical: 7-14 days
  - High: 14-30 days
  - Medium: 30-60 days
  - Low: 60-90 days

## Security Practices

### Signed Releases

All releases are cryptographically signed using Sigstore Cosign. See [Verifying Artifacts](docs/security/verifying-artifacts.md) for instructions.

### SBOM

Software Bill of Materials (SBOM) files are generated and published with each release.

### Dependency Scanning

We use `cargo-audit` and `osv-scanner` to scan dependencies for known vulnerabilities.

### Vulnerability Disclosure

Security advisories are published at: https://github.com/firestoned/kube-condition/security/advisories

## Supply Chain Security

- ✅ All releases are built in GitHub Actions (SLSA Level 3)
- ✅ Signatures recorded in Rekor transparency log
- ✅ SBOM generated for each release
- ✅ Automated dependency scanning
```

**Action Items:**
- [ ] Create `SECURITY.md`
- [ ] Configure GitHub Security Advisories
- [ ] Set up security contact email
- [ ] Document security practices in README
- [ ] Enable Dependabot security updates

---

## Implementation Timeline

### Week 1-2: SBOM Foundation
- [ ] Install and test `cargo-sbom`
- [ ] Generate SBOM locally for both crates
- [ ] Add Makefile targets for SBOM generation
- [ ] Create GitHub workflow for SBOM generation
- [ ] Test SBOM generation on development branch
- [ ] Validate SBOM format with `cyclonedx-cli`

### Week 3-4: SBOM Integration & Distribution
- [ ] Attach SBOMs to GitHub releases
- [ ] Add SBOM validation step to CI
- [ ] Set up vulnerability scanning with `osv-scanner`
- [ ] Document SBOM availability in README
- [ ] Update documentation site with SBOM information
- [ ] Test SBOM download and verification

### Week 5-6: Signing Setup
- [ ] Install and test Cosign locally
- [ ] Test signing with keypair (local only)
- [ ] Implement keyless signing in CI
- [ ] Sign SBOM files in release workflow
- [ ] Verify signatures in Rekor transparency log
- [ ] Create verification documentation

### Week 7-8: Complete Signing Implementation
- [ ] Sign `.crate` files in release workflow
- [ ] Test end-to-end verification as a user
- [ ] Write comprehensive verification docs
- [ ] Update README with verification instructions
- [ ] Create quick-start verification script
- [ ] Test all verification steps

### Week 9-10: Advanced Security (Optional)
- [ ] Implement SLSA provenance attestation
- [ ] Set up automated vulnerability scanning
- [ ] Configure Dependabot
- [ ] Create SECURITY.md
- [ ] Monitor Rekor transparency logs
- [ ] Set up security alerts

### Week 11-12: Documentation & Polish
- [ ] Complete all security documentation
- [ ] Create security section in docs site
- [ ] Write blog post on supply chain security
- [ ] Test all workflows end-to-end
- [ ] Prepare announcement of security features

---

## Success Criteria

### SBOM Requirements
- ✅ SBOM generated for every release automatically
- ✅ SBOM attached to GitHub releases
- ✅ SBOM validates against CycloneDX/SPDX specification
- ✅ SBOM includes all transitive dependencies
- ✅ Documentation explains how to consume SBOM
- ✅ Vulnerability scanning integrated

### Signing Requirements
- ✅ All release artifacts signed with Cosign (keyless)
- ✅ Signatures verifiable via keyless verification
- ✅ Signatures recorded in Rekor transparency log
- ✅ Verification instructions documented and tested
- ✅ Certificate identity bound to GitHub repository
- ✅ No long-lived secrets in CI/CD

### Documentation Requirements
- ✅ Comprehensive verification guide
- ✅ README includes verification instructions
- ✅ SECURITY.md created
- ✅ Supply chain security documented
- ✅ Troubleshooting guide for verification

### Compliance Requirements (Regulated Environments)
- ✅ SBOM meets regulatory requirements (EO 14028, EU CRA)
- ✅ Signing meets non-repudiation requirements
- ✅ Transparency log provides audit trail
- ✅ Security practices documented

---

## Compliance & Regulatory Notes

**For regulated banking environments:**

### SBOM Requirements
- **Executive Order 14028 (US)**: Federal agencies must obtain SBOM from software suppliers
- **EU Cyber Resilience Act**: Requires SBOM for software products
- **NIST SSDF**: Recommends SBOM as part of secure software development
- **Banking regulations**: Many financial institutions require SBOM for third-party software

### Signing Requirements
- **Non-repudiation**: Digital signatures provide proof of origin
- **Authenticity**: Verify software hasn't been tampered with
- **Integrity**: Detect unauthorized modifications
- **Chain of custody**: Track software provenance

### Transparency Logs
- **Auditability**: All signatures recorded immutably
- **Compliance**: Meet audit and compliance requirements
- **Forensics**: Investigate security incidents

### Internal Actions
- [ ] Review SBOM format with compliance team
- [ ] Validate signing approach with security team
- [ ] Ensure Rekor log retention meets audit requirements
- [ ] Document supply chain security in compliance artifacts
- [ ] Get approval for Sigstore/Cosign usage

---

## Cost & Resource Analysis

### Tooling Costs

| Tool | Cost | Notes |
|------|------|-------|
| cargo-sbom | Free | Open source |
| Cosign | Free | Open source |
| Sigstore infrastructure | Free | Public good infrastructure |
| GitHub Actions | Free | For public repositories |
| cyclonedx-cli | Free | Open source |
| osv-scanner | Free | Open source |

**Total Cost:** $0 for open source projects

### Time Investment

| Phase | Estimated Time | Complexity |
|-------|---------------|------------|
| SBOM Setup | 4-8 hours | Low |
| SBOM Integration | 4-8 hours | Low |
| Signing Setup | 8-12 hours | Medium |
| Signing Integration | 8-12 hours | Medium |
| Documentation | 8-16 hours | Medium |
| Testing & Verification | 4-8 hours | Low |
| **Total** | **36-64 hours** | - |

### Ongoing Maintenance

- **Weekly**: Review vulnerability scan results (15 min)
- **Monthly**: Update dependencies (1-2 hours)
- **Quarterly**: Review security practices (2-4 hours)
- **Per release**: SBOM and signing automated (0 manual time)

---

## Troubleshooting Guide

### SBOM Generation Issues

**Problem: `cargo-sbom` fails to install**

```bash
# Solution: Update Rust toolchain
rustup update stable

# OR use specific version
cargo install cargo-sbom --version 0.9.0
```

**Problem: SBOM missing transitive dependencies**

```bash
# Solution: Ensure Cargo.lock is present
cargo generate-lockfile
cargo sbom --output-format cyclonedx_json
```

**Problem: SBOM validation fails**

```bash
# Check SBOM format
cyclonedx validate --input-file kube-condition.cdx.json --fail-on-errors

# Re-generate with latest cargo-sbom
cargo install cargo-sbom --force
cargo sbom --output-format cyclonedx_json > kube-condition.cdx.json
```

### Signing Issues

**Problem: `cosign sign-blob` fails with "OIDC token not found"**

```yaml
# Solution: Ensure id-token permission in workflow
permissions:
  id-token: write  # ✅ Required
  contents: write
```

**Problem: Signature verification fails**

```bash
# Check certificate identity matches repository
cosign verify-blob \
  --bundle kube-condition.cdx.json.bundle \
  --certificate-identity-regexp "^https://github.com/YOUR_ORG/YOUR_REPO" \  # ✅ Must match
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition.cdx.json
```

**Problem: Rekor log unreachable**

```bash
# Check Rekor status
curl -s https://rekor.sigstore.dev/api/v1/log/publicKey

# Use alternative Rekor instance (if available)
cosign sign-blob --rekor-url https://rekor.example.com ...
```

### Vulnerability Scanning Issues

**Problem: `osv-scanner` reports false positives**

```bash
# Create osv-scanner.toml to ignore specific vulnerabilities
cat > osv-scanner.toml <<EOF
[[IgnoredVulns]]
id = "GHSA-xxxx-yyyy-zzzz"
reason = "Not applicable to our use case"
EOF
```

**Problem: `cargo-audit` fails on advisory database update**

```bash
# Clear advisory database cache
rm -rf ~/.cargo/advisory-db

# Re-run audit
cargo audit
```

---

## References & Resources

### Official Documentation
- [cargo-sbom documentation](https://github.com/psastras/sbom-rs)
- [CycloneDX Specification](https://cyclonedx.org/)
- [SPDX Specification](https://spdx.dev/)
- [Sigstore Cosign](https://docs.sigstore.dev/cosign/overview/)
- [Sigstore Rekor](https://docs.sigstore.dev/rekor/overview/)
- [SLSA Framework](https://slsa.dev/)
- [GitHub OIDC with Cosign](https://github.blog/2021-12-06-safeguard-container-signing-capability-actions/)

### Regulatory & Compliance
- [EO 14028 - Software Supply Chain Security](https://www.nist.gov/itl/executive-order-improving-nations-cybersecurity)
- [NIST SSDF (Secure Software Development Framework)](https://csrc.nist.gov/publications/detail/sp/800-218/final)
- [EU Cyber Resilience Act](https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act)

### Rust Ecosystem
- [Rust Supply Chain Security](https://doc.rust-lang.org/cargo/reference/registries.html)
- [RustSec Advisory Database](https://rustsec.org/)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [Tracking: crates.io signing support](https://github.com/rust-lang/rfcs/pull/3579)

### Tools
- [osv-scanner](https://google.github.io/osv-scanner/)
- [cyclonedx-cli](https://github.com/CycloneDX/cyclonedx-cli)
- [syft](https://github.com/anchore/syft)
- [grype](https://github.com/anchore/grype) (vulnerability scanner)

### Community Resources
- [Sigstore Community](https://www.sigstore.dev/)
- [OpenSSF Best Practices](https://bestpractices.coreinfrastructure.org/)
- [CNCF Supply Chain Security](https://www.cncf.io/blog/2021/12/15/supply-chain-security-best-practices/)

---

## Maintenance Checklist

### Daily
- [ ] Monitor GitHub Actions workflow status
- [ ] Check for failed SBOM generation or signing

### Weekly
- [ ] Review `osv-scanner` vulnerability reports
- [ ] Check Dependabot alerts
- [ ] Review `cargo-audit` output

### Monthly
- [ ] Update dependencies
- [ ] Review security advisories
- [ ] Update SBOM generation tools (`cargo-sbom`, etc.)
- [ ] Verify signatures in Rekor log

### Quarterly
- [ ] Review and update security documentation
- [ ] Audit signing and SBOM processes
- [ ] Review compliance with regulatory requirements
- [ ] Update SECURITY.md
- [ ] Review and update this roadmap

### On Each Release
- [ ] Verify SBOM generated successfully
- [ ] Verify all artifacts signed
- [ ] Verify signatures appear in Rekor
- [ ] Test artifact verification as end user
- [ ] Update CHANGELOG with security artifacts
- [ ] Announce security features in release notes

---

## FAQ

**Q: Why use Cosign instead of GPG?**

A: Cosign with keyless signing eliminates key management complexity, integrates seamlessly with CI/CD via OIDC, and provides transparency via Rekor. GPG requires managing long-lived keys and is harder to automate securely.

**Q: Do I need to sign artifacts if I publish to crates.io?**

A: crates.io doesn't currently verify signatures, but signing provides defense-in-depth, enables verification of GitHub release downloads, and future-proofs for when crates.io adds signature support.

**Q: What SBOM format should I use?**

A: Use CycloneDX for security-focused projects (better vulnerability mapping). Use SPDX for broader compatibility. For maximum compatibility, generate both.

**Q: How do users verify signatures without storing public keys?**

A: With keyless signing, verification uses the Sigstore public key infrastructure. Users verify via GitHub OIDC identity, not a project-specific key.

**Q: What if Sigstore goes down?**

A: Signatures include the Rekor log entry and certificate, so verification can work offline. The Sigstore infrastructure is designed for high availability, backed by Google, Red Hat, and other organizations.

**Q: How much does this increase release time?**

A: SBOM generation: ~10-30 seconds. Signing: ~5-10 seconds per artifact. Total overhead: <1 minute for typical releases.

**Q: Can I use this for private repositories?**

A: Yes, all tools work with private repositories. Keyless signing still works via GitHub OIDC. Consider whether you want signatures in the public Rekor log.

---

## Conclusion

Implementing SBOM generation and cryptographic signing provides:

✅ **Regulatory Compliance** - Meet SBOM requirements
✅ **Supply Chain Security** - Verify artifact authenticity
✅ **Transparency** - Public audit trail via Rekor
✅ **Trust** - Build confidence in your releases
✅ **Future-Proof** - Ready for crates.io signing support

**Start with SBOM** (Phase 1) - easier, immediate value
**Add Signing** (Phase 2) - stronger security guarantees
**Enhance with SLSA** (Phase 3) - best-in-class provenance

**Next Steps:**
1. Review and approve this roadmap
2. Begin Phase 1: SBOM Generation
3. Test SBOM generation locally
4. Integrate into CI/CD
5. Move to Phase 2: Signing

---

**Document Version:** 1.0
**Last Updated:** 2025-12-18
**Status:** Ready for Implementation
