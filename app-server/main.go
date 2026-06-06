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

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading config: %v\nRun with --init to initialize.\n", err)
		os.Exit(1)
	}

	fmt.Printf("Starting app-server v%s on port %d\n", Version, *portFlag)
	fmt.Printf("Data directory: %s\n", cfg.DataDir)

	srv := api.NewServer(cfg, *portFlag, Version)
	if err := srv.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "Server error: %v\n", err)
		os.Exit(1)
	}
}
