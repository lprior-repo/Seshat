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
