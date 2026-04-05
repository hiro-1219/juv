# juv - Fast, Seamless Package Manager for Julia

`juv` is a command-line tool written in Rust designed to bring a `uv` or `cargo`-like experience to the Julia programming language. It acts as a high-performance wrapper around Julia's `Pkg.jl` while implementing its own concurrent download engine to dramatically speed up environment synchronization.

## Features

- **Cargo-style CLI**: Familiar commands like `init`, `add`, `run`, `sync`, and `build`.
- **Automatic Julia Version Management**: Automatically ensures the correct Julia version is installed and used based on `Project.toml` (via `juliaup`).
- **Concurrent Downloads**: Native Rust implementation using `tokio` and `reqwest` to download and extract multiple packages simultaneously.
- **GitHub Integration**: Optimized fetching from GitHub via Tarball archives, bypassing heavy `git clone` operations for most use cases.
- **Project Compilation**: Build your project into standalone executables or fast-booting sysimages using `PackageCompiler.jl`.
- **Standard Compatibility**: Fully compatible with Julia's `Project.toml` and `Manifest.toml`.

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Juliaup](https://github.com/JuliaLang/juliaup) (required for version management)
- [Julia](https://julialang.org/downloads/)

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

### Julia Version Management
`juv` reads the `[compat]` section of your `Project.toml`:
```toml
[compat]
julia = "~1.10"
```
When you run any command, `juv` will:
1. Check if the current Julia matches.
2. If not, find a matching version in `juliaup`.
3. If not installed, automatically run `juliaup add 1.10` and use it.

### Synchronize environment
This resolves dependencies via Julia and then performs a **parallel download** of missing artifacts in Rust:
```bash
juv sync
```

### Run scripts
Runs a Julia script within the project's environment (`--project=@.`) using the correct Julia version:
```bash
juv run script.jl -- some_args
```

### Build the project
Compile your project into an app or sysimage:
```bash
# Build a standalone executable app
juv build --app --entry main.jl --output ./my_app

# Build a shared sysimage for faster startup
juv build --sysimage --output ./build
```

## Architecture

`juv` follows a hybrid approach to maximize speed and compatibility:

1. **Version Selection**: Uses `semver` in Rust to resolve `Project.toml` requirements against `juliaup` channels.
2. **Resolution**: Offloads dependency resolution to Julia's `Pkg.jl`. This ensures 100% compatibility with Julia's ecosystem and resolver logic.
3. **Synchronization**: Parses TOMLs natively in Rust. If packages are missing from the local depot, `juv` fetches them concurrently using its internal async engine.
4. **Download Strategy**:
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
