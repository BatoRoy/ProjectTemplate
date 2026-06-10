# Bato auto-update

This template is wired to publish builds to a self-hosted **bato** (MinIO) and
to **auto-update** itself from there. Every app you create from the template
inherits this — you only adjust a few identifiers per app.

## What's wired in

- `app-client/electron/updater.js` — electron-updater with a "Restart now / Later"
  prompt when a new version finishes downloading. Called from `main.js`, and only
  when the app is packaged (dev runs are never updated).
- `app-client/package.json` →
  - `dependencies.electron-updater`
  - `build.publish` — an S3 provider pointing at your MinIO server. electron-builder
    bundles this into `app-update.yml`, so the shipped app knows where to look.
  - `scripts.publish:linux` — builds the renderer, then `electron-builder --publish always`.
- `app-client/bato.json` — lets the `bato` CLI build + publish in one command.
- `Makefile` → `make publish-client`.

Version is driven by the existing `VERSION` file / `version.sh`, which already
syncs `package.json`. electron-updater reads the version from there — no extra step.

## Per-app setup (in addition to the README's find-and-replace)

When you start a new app from this template, change these so its builds land in
their own slot and update from it. **All three must agree on the path.**

| Set this                         | In                               | Example          |
| -------------------------------- | -------------------------------- | ---------------- |
| `build.publish.endpoint`         | `app-client/package.json`        | your MinIO LAN URL |
| `build.publish.path`             | `app-client/package.json`        | `apps/notes`     |
| `s3Path`                         | `app-client/bato.json`          | `apps/notes`     |
| `name`                           | `app-client/bato.json`          | `notes`          |
| `appId`                          | `app-client/package.json`        | `com.you.notes`  |

(The `name`/`appId`/`productName` find-and-replace from the README still applies.)

## Publishing a release

```bash
./version.sh bump patch        # bumps VERSION + package.json, commits, tags
export BATO_S3_ENDPOINT=http://<server>:9000
export BATO_ACCESS_KEY=bato BATO_SECRET_KEY=<minio-password>
export BATO_BUCKET=releases BATO_REGISTRY_URL=http://<server>:8080
make publish-client            # builds + uploads the AppImage to apps/<name>
```

Installed copies pick it up on next launch and prompt to restart. The GitHub
Actions release workflow is still present if you also want public GitHub releases;
it's independent of the bato path.
