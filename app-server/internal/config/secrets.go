package config

// Secrets reach a server three equal ways, in precedence order:
// CLI flags > environment variables > .env files (working directory, then
// ~/.config/<app>/.env). See BatoApps STANDARDIZATION-PLAN.md. Servers must
// never crash at startup over a missing secret — warn and let the feature
// that needs it fail at call time with a clear message.

import (
	"bufio"
	"log"
	"os"
	"path/filepath"
	"strings"
)

// LoadEnvFiles loads KEY=VALUE lines from ./.env and ~/.config/<appName>/.env
// into the process environment, never overriding variables that are already
// set — so real environment variables always beat file contents. Missing
// files are fine; .env is optional.
func LoadEnvFiles(appName string) {
	paths := []string{".env"}
	if dir, err := os.UserConfigDir(); err == nil {
		paths = append(paths, filepath.Join(dir, appName, ".env"))
	}
	for _, p := range paths {
		if err := loadEnvFile(p); err != nil && !os.IsNotExist(err) {
			log.Printf("warning: could not load %s: %v", p, err)
		}
	}
}

func loadEnvFile(path string) error {
	f, err := os.Open(path)
	if err != nil {
		return err
	}
	defer f.Close()
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		key, val, ok := strings.Cut(line, "=")
		if !ok {
			continue
		}
		key = strings.TrimSpace(strings.TrimPrefix(key, "export "))
		val = strings.Trim(strings.TrimSpace(val), `"'`)
		if _, exists := os.LookupEnv(key); !exists {
			os.Setenv(key, val)
		}
	}
	return sc.Err()
}

// Secret resolves one secret: explicit flag value first, then the env var
// (which LoadEnvFiles may have filled from a .env file).
func Secret(flagVal, envName string) string {
	if flagVal != "" {
		return flagVal
	}
	return os.Getenv(envName)
}
