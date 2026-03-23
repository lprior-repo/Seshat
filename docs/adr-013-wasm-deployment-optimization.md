# ADR 013: WASM Deployment & Optimization Strategy

## Context
The Seshat `diagram_tool` relies on a massive icon set (over 2,248 SVGs/PNGs representing ~34MB of data) for cloud provider components. To ensure all icons are immediately available without network popping or delayed rendering during the initial user experience, we elected to forcefully embed these assets directly into the compiled WASM binary.

However, embedding 34MB of binary data along with massive Base64 strings caused the raw `.wasm` artifact to balloon to ~147MB, making it extremely inefficient to deploy and serve over the web.

Additionally, using GitHub Actions to perform compilation and deployment proved unnecessarily slow due to the CI overhead of fetching dependencies, compiling the Rust crate, and managing environment transitions.

## Decisions

1. **Keep Assets Embedded**: We will continue embedding the icons directly in the WASM binary using Rust's `include_dir!()` and Base64 string conversion to guarantee zero pop-in latency for the UI.
2. **Aggressive WASM Optimization (`wasm-opt`)**: We will use Binaryen's `wasm-opt -Oz` to aggressively strip debug symbols, unused code, and aggressively shrink the WASM footprint down. This successfully brings the binary from ~147MB down to ~70MB.
3. **Brotli Compression (`brotli -c -q 11`)**: We will compress the resulting WASM binary with Brotli at maximum quality to significantly reduce the network payload size. GitHub Pages natively supports serving Brotli compressed assets, which brings the effective download payload to users down to ~52MB.
4. **Local Deployment Pipeline via Moon**: We removed GitHub Actions for deployment to GitHub Pages. Instead, deployment will be handled strictly via a local `moon` task (`moon run :deploy-pages`).
   - The task locally builds the WASM file, runs optimizations, and initializes a clean Git history within the `dist/public` folder.
   - It force-pushes this optimized output directly to the remote `gh-pages` branch.
   - This keeps the large `70MB` WASM blobs out of the `main` branch Git history, preventing the repository from rapidly bloating in size, while ensuring deployments take seconds instead of minutes.

## Consequences

### Positive
- Deployments to GitHub pages are near-instant and no longer rely on waiting for GitHub Actions.
- The Git history of the `main` branch stays clean and free of massive binary blobs.
- End-users receive all 2,248 icons instantly upon loading the application with zero layout shift or staggered network requests.

### Negative
- The initial load payload is ~52MB. While Brotli helps, this is still significantly larger than a standard web application, meaning users on slow connections will face a long initial loading screen.
- Developers must have `brotli` installed on their machine to run the `deploy-pages` moon task locally (the task handles fetching `wasm-opt` automatically if missing).