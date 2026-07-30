# Container image for the Go backend — the `bato deploy` path.
#
# Only needed if this app's server is a *daemon* (runs on the Bato host,
# independent of any client). A session-bound server, spawned and killed by the
# Electron client, never gets containerised — see BUNDLED-SERVICES.md.
#
# Build context is this directory:
#   docker build -t <registry>/<name>:<version> app-server/
# though in practice `bato deploy build` does it for you.

# ── Stage 1: build ──────────────────────────────────────────
FROM golang:1.22-alpine AS build
WORKDIR /src
COPY go.mod ./
# No external modules in the template. Copy go.sum too if you ever add any.
RUN go mod download 2>/dev/null || true
COPY . .
ARG VERSION=docker
RUN CGO_ENABLED=0 go build -ldflags "-s -w -X main.Version=${VERSION}" -o /app-server .

# ── Stage 2: runtime ────────────────────────────────────────
FROM alpine:3.20
RUN apk add --no-cache ca-certificates wget && adduser -D -u 10001 app
WORKDIR /app
COPY --from=build /app-server /usr/local/bin/app-server

# The port INSIDE the container. It is deliberately fixed and unrelated to the
# host port: the service's real port is its PORTS.md assignment, declared as
# `port` in bato.json, and `bato deploy` publishes that onto this one via
# deploy.targetPort. Keeping the inside constant means the image doesn't need
# rebuilding to move a service to a different host port.
ENV APP_PORT=8080

# Where the server keeps anything it expects to find after a redeploy. It must
# come from the environment: a container has no ~/.config/app/config.json, and
# a server that insists on one crash-loops on "Run with --init" while Dokploy
# reports the deploy as done. Pair it with a *named* volume in bato.json's
# deploy.volumes — the VOLUME line below alone gets an anonymous volume, and
# Swarm hands each new task a fresh one.
ENV APP_DATA_DIR=/data

RUN mkdir -p /data && chown app:app /data
USER app
VOLUME ["/data"]
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=4s --start-period=5s \
  CMD wget -qO- "http://localhost:${APP_PORT}/api/health" >/dev/null 2>&1 || exit 1

# sh -c so ${APP_PORT} expands; exec so the server stays PID 1 and still
# receives SIGTERM on `docker stop`.
ENTRYPOINT ["sh", "-c", "exec app-server --port \"$APP_PORT\""]
