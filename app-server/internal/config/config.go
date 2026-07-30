package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

const configDir = ".config/app"
const configFile = "config.json"

// Config holds backend settings. Add fields as your app grows.
type Config struct {
	DataDir string `json:"data_dir"`
}

func configPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, configDir, configFile), nil
}

// Init writes a fresh config pointing at dataDir.
func Init(dataDir string) error {
	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}
	dir := filepath.Join(home, configDir)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("creating config dir: %w", err)
	}

	cfg := Config{DataDir: dataDir}
	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return err
	}

	path := filepath.Join(dir, configFile)
	return os.WriteFile(path, data, 0644)
}

// DataDir resolves where the server keeps things it expects to find later, in
// the suite's standard precedence: flag > env > config.json > default.
//
// The env step is what makes this server containerisable. A container has no
// ~/.config/app/config.json and must not need one — it gets APP_DATA_DIR=/data
// from the Dockerfile, matched by the named volume in deploy.volumes. Making
// config.json mandatory is how a deployed container ends up crash-looping on
// "Run with --init" while Dokploy reports the deploy as done.
//
// The config.json step is still here for the tarball + systemd path, where
// `--init` records the data directory once and the unit file carries no flags.
func DataDir(flagVal string) string {
	if flagVal != "" {
		return flagVal
	}
	if v := os.Getenv("APP_DATA_DIR"); v != "" {
		return v
	}
	if cfg, err := Load(); err == nil && cfg.DataDir != "" {
		return cfg.DataDir
	}
	return "./data"
}

// Load reads the config from disk. Prefer DataDir() on the start path — a
// missing config file is a normal state for a container, not an error.
func Load() (*Config, error) {
	path, err := configPath()
	if err != nil {
		return nil, err
	}
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("reading config: %w", err)
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("parsing config: %w", err)
	}
	return &cfg, nil
}
