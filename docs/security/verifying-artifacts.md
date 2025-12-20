# Verifying Signed Artifacts

This guide explains how to verify the authenticity and integrity of kube-condition releases using Cosign.

## Prerequisites

Install Cosign:

```bash
# macOS
brew install cosign

# Linux
COSIGN_VERSION=$(curl -s https://api.github.com/repos/sigstore/cosign/releases/latest | grep tag_name | cut -d '"' -f 4)
wget "https://github.com/sigstore/cosign/releases/download/${COSIGN_VERSION}/cosign-linux-amd64"
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign

# Verify installation
cosign version
```

## What Gets Signed

We sign all release artifacts using **Cosign keyless signing** with GitHub OIDC:

- **SBOM files** (`.cdx.json`) - Software Bill of Materials
- **Crate archives** (`.crate`) - Rust package files
- **Container images** (if published)

Each signed artifact has a corresponding `.bundle` file containing:
- The signature
- The signing certificate (from Fulcio)
- Rekor transparency log entry
- All metadata needed for verification

This is a **self-contained verification artifact** - no external key needed!

## Verify SBOM Signature

Download the SBOM and signature bundle from the [Releases page](https://github.com/YOUR_ORG/kube-condition/releases).

```bash
# Download SBOM and bundle (replace v0.1.0 with actual version)
curl -sL https://github.com/YOUR_ORG/kube-condition/releases/download/v0.1.0/kube-condition.cdx.json -o kube-condition.cdx.json
curl -sL https://github.com/YOUR_ORG/kube-condition/releases/download/v0.1.0/kube-condition.cdx.json.bundle -o kube-condition.cdx.json.bundle

# Verify using keyless signing (GitHub OIDC)
cosign verify-blob \
  --bundle kube-condition.cdx.json.bundle \
  --certificate-identity-regexp "^https://github.com/YOUR_ORG/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition.cdx.json
```

**Expected output:**
```
Verified OK
```

**What This Verifies:**
- ✅ The SBOM was signed by a GitHub Actions workflow
- ✅ The workflow ran in the `YOUR_ORG/kube-condition` repository
- ✅ The signature was recorded in the Rekor transparency log
- ✅ The file has not been tampered with since signing

## Verify .crate File Signature

```bash
# Download .crate file and bundle (replace version)
curl -sL https://github.com/YOUR_ORG/kube-condition/releases/download/v0.1.0/kube-condition-0.1.0.crate -o kube-condition-0.1.0.crate
curl -sL https://github.com/YOUR_ORG/kube-condition/releases/download/v0.1.0/kube-condition-0.1.0.crate.bundle -o kube-condition-0.1.0.crate.bundle

# Verify signature
cosign verify-blob \
  --bundle kube-condition-0.1.0.crate.bundle \
  --certificate-identity-regexp "^https://github.com/YOUR_ORG/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition-0.1.0.crate
```

## Understanding Keyless Signing

When we use keyless signing, the signature is bound to a **certificate identity**:

```
--certificate-identity-regexp "^https://github.com/YOUR_ORG/kube-condition"
```

This means:
- Only workflows from this repository can produce valid signatures
- Attackers cannot forge signatures (even with access to the artifact)
- The signature is cryptographically bound to the GitHub repository
- No long-lived private keys to manage or compromise

## Transparency Log (Rekor)

All signatures are recorded in the public **Rekor transparency log**.

You can search for signatures:

```bash
# Install rekor-cli (Go required)
go install github.com/sigstore/rekor/cmd/rekor-cli@latest

# Search Rekor log for signatures from our repository
rekor-cli search --email noreply@github.com
```

This provides:
- **Tamper evidence**: Signatures cannot be backdated or removed
- **Auditability**: Anyone can verify when artifacts were signed
- **Non-repudiation**: Proof that the artifact came from our CI/CD pipeline

## Troubleshooting

### "signature verification failed"

**Possible causes:**
1. The file was modified after signing
2. The signature bundle doesn't match the file
3. The certificate identity doesn't match

**Solution:**
- Re-download both the artifact and `.bundle` file
- Ensure you're using the correct `--certificate-identity-regexp` for the repository

### "certificate identity does not match"

**Cause:** The `--certificate-identity-regexp` doesn't match the repository that signed the artifact.

**Solution:**
- Update the command to use the correct repository name
- For kube-condition: `"^https://github.com/YOUR_ORG/kube-condition"`

### "connection to rekor.sigstore.dev failed"

**Cause:** Network connectivity issue or Rekor service unavailable.

**Solution:**
- Check your internet connection
- Verification requires access to the Rekor transparency log
- Check Sigstore status: https://status.sigstore.dev/

### "Failed to verify signature"

**Possible causes:**
1. Using an old version of Cosign (< 2.0)
2. The artifact was not signed
3. Network issues preventing access to Rekor

**Solution:**
- Update Cosign: `brew upgrade cosign` (macOS) or re-download latest version
- Ensure artifact is from an official release
- Check network connectivity to rekor.sigstore.dev

## Security Considerations

### ✅ Best Practices

- **Always verify signatures before using release artifacts**
- **Verify the certificate identity matches the expected repository**
- **Check the transparency log for unusual signing activity**
- **Use the latest version of Cosign for security fixes**
- **Report suspicious signatures to security@YOUR_DOMAIN.com**

### ⚠️ What Signatures DON'T Guarantee

- **Not a code review**: Signatures prove artifacts came from our CI/CD, not code quality
- **Not vulnerability-free**: Check SBOM for known vulnerabilities separately
- **Not license compliance**: Review LICENSE file and SBOM for dependency licenses

## Advanced: Verify Signature in Rekor Log

```bash
# Get signature UUID from verification
cosign verify-blob \
  --bundle kube-condition.cdx.json.bundle \
  --certificate-identity-regexp "^https://github.com/YOUR_ORG/kube-condition" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  kube-condition.cdx.json 2>&1 | grep "tlog entry"

# Query Rekor for details
rekor-cli get --uuid <UUID>
```

This shows:
- Exact timestamp of signing
- Certificate details
- Payload hash
- Rekor log index

## Automation: Verify in CI/CD

Add verification to your CI/CD pipeline:

```yaml
- name: Download and verify SBOM
  run: |
    # Download SBOM and bundle
    curl -sL https://github.com/YOUR_ORG/kube-condition/releases/latest/download/kube-condition.cdx.json -o kube-condition.cdx.json
    curl -sL https://github.com/YOUR_ORG/kube-condition/releases/latest/download/kube-condition.cdx.json.bundle -o kube-condition.cdx.json.bundle

    # Verify signature
    cosign verify-blob \
      --bundle kube-condition.cdx.json.bundle \
      --certificate-identity-regexp "^https://github.com/YOUR_ORG/kube-condition" \
      --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
      kube-condition.cdx.json

    echo "✓ SBOM signature verified successfully"
```

## Questions?

If you have questions about artifact verification:
- **Documentation**: https://docs.sigstore.dev/cosign/overview/
- **Issues**: https://github.com/YOUR_ORG/kube-condition/issues
- **Security**: security@YOUR_DOMAIN.com (for security-sensitive questions)

---

**Last Updated:** 2025-12-18
**Cosign Version:** 2.4.1+
**Sigstore Docs:** https://docs.sigstore.dev/
