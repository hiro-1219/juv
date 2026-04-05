# juv - Fast, Seamless Package Manager for Julia

`juv` is a command-line tool written in Rust designed to bring a `uv` or `cargo`-like experience to the Julia programming language. It acts as a high-performance wrapper around Julia's `Pkg.jl` while implementing its own concurrent download engine to dramatically speed up environment synchronization.

## Features

- **Cargo-style CLI**: Familiar commands like `init`, `add`, `run`, and `sync`.
- **Concurrent Downloads**: Native Rust implementation using `tokio` and `reqwest` to download and extract multiple packages simultaneously.
- **GitHub Integration**: Optimized fetching from GitHub via Tarball archives, bypassing heavy `git clone` operations for most use cases.
- **Standard Compatibility**: Fully compatible with Julia's `Project.toml` and `Manifest.toml`. It manages the standard Julia Depot (`~/.julia`).
- **Isolated Execution**: Automatically sets up the environment to run scripts in the context of your local project.

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Julia](https://julialang.org/downloads/)
- [Juliaup](https://github.com/JuliaLang/juliaup) (recommended)

### Build
Clone this repository and build the binary:
```bash
cargo build --release
```
The binary will be available at `./target/release/juv`. You can symlink it to your path for convenience.

## Usage

### Initialize a new project
```bash
# In an empty directory
juv init
```

### Add packages
Supports both registry names and GitHub URLs:
```bash
juv add JSON
juv add https://github.com/JuliaLang/Example.jl
```

### Synchronize environment
This resolves dependencies via Julia and then performs a **parallel download** of missing artifacts in Rust:
```bash
juv sync
```

### Run scripts
Runs a Julia script within the project's environment (`--project=@.`):
```bash
juv run script.jl -- some_args
```

## Architecture

`juv` follows a hybrid approach to maximize speed and compatibility:

1. **Resolution**: Offloads dependency resolution to Julia's `Pkg.jl`. This ensures 100% compatibility with Julia's ecosystem and resolver logic.
2. **Synchronization**: Parses `Manifest.toml` in Rust. If packages are missing from the local depot, `juv` fetches them concurrently using its internal async engine.
3. **Download Strategy**:
   - **General Registry Packages**: Fetched from `pkg.julialang.org`.
   - **GitHub Packages**: Fetched via the GitHub Archive API (`.tar.gz`).
   - **Other Git Repos**: Fallback to shallow `git clone`.

## Development

To run the integration tests in a sandbox environment:
```bash
bash test_juv.sh
```

## License
MIT / Apache-2.0
