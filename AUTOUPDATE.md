# Bato auto-update

> **Electron flavor only.** Everything here is `electron-updater`, which works because
> `apps/` is the one anonymously readable bucket prefix and ships a `latest-linux.yml`.
> A QML client publishes as `type: "desktop"` under `desktop/`, which is neither, so it
> has no self-update path at all: its About dialog asks the registry for the latest
> version and shows `bato install <slug>`. `new-app.sh --qml` deletes this file.


This template is wired to publish builds to a self-hosted **bato** (MinIO) and
to **auto-update** itself from there. Every app you create from the template
inherits this — you only adjust a few identifiers per app.

## What's wired in

- `app-client/electron/updater.js` — electron-updater streaming status to the
  renderer over IPC (`update:status` events with phases `checking` / `available` /
  `none` / `downloading {percent}` / `downloaded {version}` / `error`, plus
  `update:check` and `update:restart` invokes). Called from `main.js`; the IPC
  surface exists in dev too, but updates only actually run when packaged.
- In-app UI: the sidebar footer shows download progress and a
  "Restart to update vX" button when ready (`hooks/useUpdateStatus.ts` +
  `components/Sidebar.tsx`), and the About dialog has a "Check for updates"
  button. A downloaded update also installs automatically on quit. To preview
  the UI in dev, fire the mock event from DevTools:

  ```js
  window.dispatchEvent(new CustomEvent('mock:update-status', { detail: { phase: 'downloading', percent: 40 } }))
  window.dispatchEvent(new CustomEvent('mock:update-status', { detail: { phase: 'downloaded', version: '9.9.9' } }))
  ```
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

Installed copies pick it up on next launch and show the in-app restart button. The GitHub
Actions release workflow is still present if you also want public GitHub releases;
it's independent of the bato path.
