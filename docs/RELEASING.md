# Releasing Vox

Releases are created by `.github/workflows/release.yml` when a semantic-version tag
is pushed. The workflow builds a universal macOS bundle and DMG, creates signed
updater artifacts plus `latest.json`, and opens a draft GitHub release.

## One-time repository setup

Generate an updater key once with the Tauri CLI and keep the private file outside
the repository:

```bash
npm run tauri signer generate -- -w /secure/path/vox-updater.key
```

The public key belongs in `src-tauri/tauri.conf.json`. Add the complete private key
contents as the `TAURI_SIGNING_PRIVATE_KEY` GitHub Actions secret. If the key has a
password, add it as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; an unencrypted key leaves
that secret unset.

Back up the private key in an encrypted credential vault. Losing it means existing
installations cannot trust updates signed by a replacement key. Never commit it,
paste it into an issue, or attach it to a release.

For Apple Developer ID signing and notarization, configure these optional secrets:

- `APPLE_CERTIFICATE` (base64-encoded `.p12`)
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD` (an app-specific password)
- `APPLE_TEAM_ID`

Updater signatures and Apple code signatures solve different trust problems. The
updater key is required by the workflow; Apple credentials are needed to avoid the
first-launch Gatekeeper warning for downloaded builds.

## Release checklist

1. Confirm `master` CI is green and the worktree is clean.
2. Update the version consistently in `package.json`, `src-tauri/Cargo.toml`, and
   `src-tauri/tauri.conf.json`.
3. Move release notes from `CHANGELOG.md` under the matching version.
4. Run the local validation commands from `README.md`.
5. Commit and push the version bump.
6. Create and push the tag:

   ```bash
   git tag -a v0.1.0 -m "Vox 0.1.0"
   git push origin v0.1.0
   ```

7. Inspect the workflow artifacts and `latest.json` URLs in the draft release.
8. Test the DMG on a second macOS account or machine, including microphone,
   Accessibility fallback, model download, dictation, and update discovery.
9. Edit the generated release notes as needed and publish the draft.

## Local bundle smoke test

With the updater signing key available:

```bash
TAURI_SIGNING_PRIVATE_KEY=/secure/path/vox-updater.key \
  npm run tauri build -- --debug --bundles app,dmg
```

Release and universal builds are intentionally left to GitHub's macOS runner so the
result is reproducible and has access to configured repository secrets.
