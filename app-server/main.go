package main

import (
	"flag"
	"fmt"
	"os"

	"app-server/internal/api"
	"app-server/internal/config"
)

func main() {
	initFlag    := flag.Bool("init", false, "Initialize the config with the current directory as the data directory")
	portFlag    := flag.Int("port", 8080, "Port to listen on")
	dataDirFlag := flag.String("data-dir", "", "Where to keep persistent data (default: $APP_DATA_DIR, else the --init config, else ./data)")
	versionFlag := flag.Bool("version", false, "Print version and exit")
	flag.Parse()

	if *versionFlag {
		fmt.Printf("app-server %s\n", Version)
		return
	}

	if *initFlag {
		cwd, err := os.Getwd()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error getting working directory: %v\n", err)
			os.Exit(1)
		}
		if err := config.Init(cwd); err != nil {
			fmt.Fprintf(os.Stderr, "Error initializing: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("Initialized config with data directory %s\n", cwd)
		fmt.Println("Run 'app-server' (without --init) to start the server.")
		return
	}

	// Fill the environment from optional .env files (precedence: flags > env >
	// .env). Secrets belong there or in `bato secrets`, never in the JSON config.
	// This runs before DataDir so a .env can supply APP_DATA_DIR.
	config.LoadEnvFiles("app")

	dataDir := config.DataDir(*dataDirFlag)
	if err := os.MkdirAll(dataDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "Error creating data directory %s: %v\n", dataDir, err)
		os.Exit(1)
	}

	// Identity comes from bato-auth. BATO_AUTH_URL is where this server verifies
	// requests (the host IP works from inside a container); BATO_AUTH_PUBLIC_URL
	// is where browsers are sent to sign in. With BATO_AUTH_URL unset the server
	// binds 127.0.0.1 — a deployed container without it is unreachable on
	// purpose, rather than serving unauthenticated.
	authURL := os.Getenv("BATO_AUTH_URL")
	mode := fmt.Sprintf("bato-auth at %s", authURL)
	if authURL == "" {
		mode = "loopback-only dev mode (BATO_AUTH_URL not set)"
	}

	fmt.Printf("app-server %s listening on :%d (data: %s) — %s\n", Version, *portFlag, dataDir, mode)

	srv := api.NewServer(dataDir, *portFlag, Version, authURL)
	if err := srv.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "Server error: %v\n", err)
		os.Exit(1)
	}
}
