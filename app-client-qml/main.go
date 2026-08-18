// Command app is the QML client's host process.
//
// It is the structural twin of app-client/electron/main.js: it does not draw the
// window (Main.qml's ApplicationWindow does), but it owns everything around it —
// finding the Qt runtime, resolving the QML, deciding whether this session gets a
// bundled backend, serving the native bridge, and outliving none of it.
//
// Unlike BatoAI's `batoai ui`, this does NOT syscall.Exec into qml6. BatoAI's
// dashboard has no children to look after, so replacing the process is free and
// tidier. Here the host owns the native bridge and (usually) a server child, so
// it has to stay alive to be their parent — exactly as Electron's main process
// does. The cost is one small Go process in the tree.
//
// Subcommands:
//
//	app            open the UI (default)
//	app ui         same, explicitly — what bato.json's launcher could use
//	app doctor     report system dependencies and where config lives
//	app --version  print the version
package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"app-client-qml/internal/appdirs"
	"app-client-qml/internal/backend"
	"app-client-qml/internal/deps"
	"app-client-qml/internal/host"
	"app-client-qml/internal/native"
)

func main() {
	args := os.Args[1:]

	cmd := "ui"
	if len(args) > 0 && !strings.HasPrefix(args[0], "-") {
		cmd, args = args[0], args[1:]
	}

	// --version/-V/--help work in any position a user is likely to try.
	for _, a := range os.Args[1:] {
		switch a {
		case "--version", "-V":
			fmt.Printf("%s %s\n", identity.productName, Version)
			return
		case "--help", "-h":
			usage()
			return
		}
	}

	var err error
	switch cmd {
	case "ui":
		err = cmdUI(args)
	case "doctor":
		err = cmdDoctor()
	case "help":
		usage()
	default:
		err = fmt.Errorf("unknown command %q — try `%s help`", cmd, strings.ToLower(identity.productName))
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "✗ %v\n", err)
		os.Exit(1)
	}
}

func usage() {
	name := identity.slug
	fmt.Printf(`%s %s

Usage:
  %s [ui]        open the UI (default)
  %s doctor      check system dependencies
  %s --version   print the version

Environment:
  APP_QML_BIN         path to the qml6 runtime, if not on PATH
  APP_QML_DIR         QML import root, overriding the search paths
  APP_SERVER_BIN      path to the bundled server binary
  APP_NO_BUNDLED_SERVER=1
                      never spawn a bundled server, even with no saved
                      backendUrl — for developing against `+"`make dev-server`"+`
`, identity.productName, Version, name, name, name)
}

// cmdUI is the whole launch sequence, in the order the pieces depend on each
// other.
func cmdUI(extraArgs []string) error {
	runtime, err := host.FindRuntime()
	if err != nil {
		return err
	}
	importRoot, mainQML, err := host.ResolveQML()
	if err != nil {
		return err
	}

	dirs := appdirs.New(identity.slug)
	settings := dirs.LoadSettings()

	// The native bridge is unconditional: settings, notifications and file I/O
	// are needed whether or not there is a bundled server.
	bridge, err := native.Start(dirs, identity.productName)
	if err != nil {
		return fmt.Errorf("could not start the native bridge: %w", err)
	}
	defer bridge.Stop()

	inject := map[string]string{
		"APP_NATIVE_URL":   bridge.URL,
		"APP_NATIVE_TOKEN": bridge.Token,
		"APP_NAME":         identity.productName,
		"APP_SLUG":         identity.slug,
		"APP_VERSION":      Version,
	}

	// Forwarded so About can ask the registry whether a newer release exists. A
	// `desktop` release cannot self-update, so detecting is all it can do — see
	// AboutDialog.qml. Absent means the check is skipped, not failed.
	if v := os.Getenv("BATO_REGISTRY_URL"); v != "" {
		inject["APP_REGISTRY_URL"] = v
	}

	// The bundled-vs-daemon decision. A saved backendUrl means the user has
	// chosen a server — pointing at an installed 42xxx daemon, possibly on
	// another machine — so spawning a second one here would be both wasteful and
	// confusing about which one the UI is actually talking to.
	//
	// Note this is stricter than BUNDLED-SERVICES.md's Electron guidance, where
	// the bundled server always wins because those apps delete the ServerUrlCard.
	// This template keeps the card, so the setting has to win.
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	switch {
	case settings.BackendURL != "":
		fmt.Printf("→ using the configured server at %s\n", settings.BackendURL)
		inject["APP_BACKEND_URL"] = settings.BackendURL

	case os.Getenv("APP_NO_BUNDLED_SERVER") == "1":
		fmt.Println("→ APP_NO_BUNDLED_SERVER=1 — not spawning a server")

	default:
		srv, err := backend.Start(ctx)
		if err != nil {
			// Not fatal, by suite rule: an unreachable backend must never block
			// the UI. The window opens, shows itself as offline, and the Server
			// section in App Options is right there to point somewhere else.
			fmt.Fprintf(os.Stderr, "⚠ no bundled server this session (%v)\n", err)
		} else {
			fmt.Printf("→ bundled server on %s\n", srv.URL)
			inject["APP_BACKEND_URL"] = srv.URL
			defer srv.Stop()
		}
	}

	cmd := host.Command(runtime, importRoot, mainQML, inject, extraArgs)
	cmd.Env = host.Env(inject)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("could not start %s: %w", runtime, err)
	}

	// Closing the window is a normal exit; so is Ctrl-C, which must take the
	// whole tree down rather than orphan the UI.
	done := make(chan error, 1)
	go func() { done <- cmd.Wait() }()

	select {
	case err := <-done:
		return err
	case <-ctx.Done():
		_ = cmd.Process.Signal(syscall.SIGTERM)
		return <-done
	}
}

func cmdDoctor() error {
	fmt.Printf("%s %s\n\n", identity.productName, Version)

	missingRequired := false
	for _, t := range deps.Check() {
		switch {
		case t.Present && t.Path != "":
			fmt.Printf("  ok   %-26s %s\n", t.Name, t.Path)
		case t.Present:
			fmt.Printf("  ok   %-26s installed\n", t.Name)
		case t.Required:
			missingRequired = true
			fmt.Printf("  MISS %-26s required — %s\n", t.Name, t.Install)
			fmt.Printf("       %-26s %s\n", "", t.Reason)
		default:
			fmt.Printf("  --   %-26s optional — %s\n", t.Name, t.Install)
			fmt.Printf("       %-26s %s\n", "", t.Reason)
		}
	}

	fmt.Println()
	if root, main, err := host.ResolveQML(); err == nil {
		fmt.Printf("  ok   %-26s %s\n", "QML import root", root)
		fmt.Printf("  ok   %-26s %s\n", "entry point", main)
	} else {
		missingRequired = true
		fmt.Printf("  MISS %-26s %v\n", "QML", err)
	}

	if bin := backend.FindBinary(); bin != "" {
		fmt.Printf("  ok   %-26s %s\n", "bundled server", bin)
	} else {
		fmt.Printf("  --   %-26s none found (the UI will need a configured server URL)\n", "bundled server")
	}

	dirs := appdirs.New(identity.slug)
	fmt.Println()
	fmt.Printf("  settings (host)   %s\n", dirs.SettingsPath())
	fmt.Printf("  preferences (UI)  %s\n", dirs.UIConfPath())
	if url := dirs.LoadSettings().BackendURL; url != "" {
		fmt.Printf("  backend           %s (configured — no bundled server will be spawned)\n", url)
	} else {
		fmt.Printf("  backend           bundled, on a free port each session\n")
	}

	if missingRequired {
		return fmt.Errorf("required dependencies are missing")
	}
	return nil
}
