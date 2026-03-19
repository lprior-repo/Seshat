package main

import (
	"context"
	"fmt"

	"dagger/seshat/internal/dagger"
)

type Seshat struct{}

func (m *Seshat) baseContainer(src *dagger.Directory) *dagger.Container {
	cargoCache := dag.CacheVolume("cargo-registry")
	cargoGitCache := dag.CacheVolume("cargo-git")
	moonCache := dag.CacheVolume("moon-cache")
	kaniCache := dag.CacheVolume("kani-cache")

	return dag.Container().
		From("archlinux:base").
		WithMountedDirectory("/src", src).
		WithWorkdir("/src").
		WithExec([]string{"bash", "-c", `
			pacman -Sy --noconfirm rustup cargo nodejs npm git pkg-config wayland && \
			rustup default stable && \
			curl -fsSL https://mise.run | bash && \
			ln -s ~/.local/bin/mise /usr/local/bin/mise && \
			cd /src && mise trust -y && \
			cargo install --locked kani-verifier && \
			cargo kani setup
		`}).
		WithMountedCache("/root/.cargo/registry", cargoCache).
		WithMountedCache("/root/.cargo/git", cargoGitCache).
		WithMountedCache("/root/.moon", moonCache).
		WithMountedCache("/root/.kani", kaniCache)
}

func (m *Seshat) Check(ctx context.Context, src *dagger.Directory) error {
	ctr := m.baseContainer(src).
		WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo check"})

	_, err := ctr.Stdout(ctx)
	return err
}

func (m *Seshat) Fmt(ctx context.Context, src *dagger.Directory) error {
	ctr := m.baseContainer(src).
		WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo fmt --check"})

	_, err := ctr.Stdout(ctx)
	return err
}

func (m *Seshat) Clippy(ctx context.Context, src *dagger.Directory) error {
	ctr := m.baseContainer(src).
		WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo clippy"})

	_, err := ctr.Stdout(ctx)
	return err
}

func (m *Seshat) Test(ctx context.Context, src *dagger.Directory) error {
	ctr := m.baseContainer(src).
		WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo test"})

	_, err := ctr.Stdout(ctx)
	return err
}

func (m *Seshat) Kani(ctx context.Context, src *dagger.Directory) (string, error) {
	ctr := m.baseContainer(src)

	// Run Kani on canvas_math (pure math functions, fully verifiable)
	// Note: Other crates (diagram_models, canvas_domain, diagram_tool) have data structures
	// (im::HashMap, getrandom syscall) that Kani struggles with
	out, err := ctr.WithExec([]string{"bash", "-c", `
		eval "$(/root/.local/bin/mise activate bash)" && \
		export PATH="$HOME/.cargo/bin:$PATH" && \
		cd /src && \
		echo "=== Running Kani on canvas_math ===" && \
		cargo kani -p canvas_math
	`}).Stdout(ctx)

	if err != nil {
		return fmt.Sprintf("Kani verification failed:\n%s", out), err
	}

	return fmt.Sprintf("Kani verification results:\n%s", out), nil
}

func (m *Seshat) Ci(ctx context.Context, src *dagger.Directory) (string, error) {
	var output string

	ctr := m.baseContainer(src)

	// Run check
	out, err := ctr.WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo check"}).Stdout(ctx)
	if err != nil {
		return out, err
	}
	output += "CHECK PASSED\n" + out

	// Run fmt
	out, err = ctr.WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo fmt --check"}).Stdout(ctx)
	if err != nil {
		return out, err
	}
	output += "FMT PASSED\n" + out

	// Run clippy
	out, err = ctr.WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo clippy"}).Stdout(ctx)
	if err != nil {
		return out, err
	}
	output += "CLIPPY PASSED\n" + out

	// Run tests
	out, err = ctr.WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo test"}).Stdout(ctx)
	if err != nil {
		return out, err
	}
	output += "TEST PASSED\n" + out

	// Run Kani on canvas_math only
	out, err = ctr.WithExec([]string{"bash", "-c", "eval \"$(/root/.local/bin/mise activate bash)\" && export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /src && cargo kani -p canvas_math"}).Stdout(ctx)
	if err != nil {
		return output + "KANI FAILED\n" + out, err
	}
	output += "KANI PASSED\n" + out

	return output, nil
}
