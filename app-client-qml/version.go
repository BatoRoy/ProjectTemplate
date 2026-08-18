package main

// Overwritten at build time with -ldflags="-X main.Version=$(VERSION)" and kept
// in sync with the repo's VERSION file by version.sh.
var Version = "0.3.0"
