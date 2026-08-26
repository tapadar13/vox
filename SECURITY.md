# Security Policy

## Supported versions

Security fixes are applied to the latest release of Vox. Upgrade to the newest
published version before reporting a problem that may already be resolved.

## Reporting a vulnerability

Please use GitHub's private vulnerability-reporting feature for this repository.
Do not open a public issue containing an exploit, updater signing material,
sensitive transcript content, or local filesystem details.

Include the Vox version, macOS version, hardware architecture, reproduction steps,
and the security impact. Remove personal transcript text and credentials from logs
before attaching them.

## Security boundaries

- Model files are downloaded only from curated HTTPS URLs and are accepted only
  after their SHA-256 digest matches the built-in registry.
- Application updates are accepted only when their signature matches the updater
  public key embedded in the app.
- The updater private key must remain outside the repository and be stored as a
  protected GitHub Actions secret.
- The frontend uses a restrictive content security policy and narrowly scoped
  Tauri capabilities.
- Audio is processed in memory and is not persisted.

Vox does not attempt to protect transcript history from another process or user
that already has access to the same macOS account. Use macOS FileVault and a strong
login password when local-at-rest confidentiality is required.

## Release provenance

Official artifacts are attached to releases in
[`tapadar13/vox`](https://github.com/tapadar13/vox). Verify that an updater artifact
is accompanied by its signature and `latest.json` metadata. macOS code signing and
notarization are separate from Vox's updater signature and require Apple Developer
credentials.
