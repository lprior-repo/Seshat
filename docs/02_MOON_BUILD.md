# Build Pipeline: Moon

## Absolute Verification
To ensure no cached success masks a subtle regression, always run:
```bash
moon run :ci-hardening --force
```

## Hardening Pipeline

The hardening release path is explicit and ordered:

```bash
moon run :check --force
moon run :test --force
moon run :clippy --force
moon run :e2e-smoke --force
moon run :e2e-full --force
```

## WASM Build & Deployment

For testing the production WASM build locally, we use an optimization pipeline that mimics the production environment (including Binaryen `wasm-opt` and `brotli` compression):

```bash
moon run :bundle-web-optimized
```

To automatically build, optimize, compress, and force-push the WASM binary to GitHub Pages directly from your local machine, run:

```bash
moon run :deploy-pages
```
