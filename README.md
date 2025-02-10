<div align="center">
  <h1>Andromeda</h1>

  <img src="https://res.cloudinary.com/dbulfrlrz/image/upload/v1693233221/static/logos/proton-logo_z7innb.svg" width="220" />

  <h3>
    <strong>"In the cosmic ballet of Andromeda's radiance, a delicate web of technologies emerges, crafted from the resilient strands of Rust."</strong>
  </h3>

  <h4>
    <a href="https://proton.me/">Proton Home</a>
    <span> | </span>
    <a href="https://docs.rs/andromeda">Documentation</a>
  </h4>
</div>

## About

The `andromeda` libraries aims to provide logical blocks to build a privacy-focused, cross-platform, self-custody Bitcoin, integrated in Proton's ecosystem

## Architecture

The project is split up into several crates in the `/crates` directory:

- [`api`](./crates/api): Contains an api client to call Proton Wallet backend HTTP API
- [`bitcoin`](./crates/bitcoin): A library that provides utilities to use bitcoin on the 1rst layer such as chain syncing, transactions/balance/utxos retrieving, address generating and obviously transaction building, signing and broadcasting.
- [`wasm`](./crates/wasm): Relevant interfaces to WASM (_should be migrated to its own repo_)

## External dependencies

- [`bdk_wallet`](https://docs.rs/bdk_wallet/)

## How to build WebAssembly package

### On macOS
This chapter outlines the steps for building the WebAssembly (Wasm) package on macOS.

You will need to install the following tools:
* Xcode Command Line Tools
* [Homebrew](https://brew.sh): macOS package manager
* gcc: GNU Compiler Collection
* llvm: Includes clang, the front-end compiler for WebAssembly
* lld: The LLVM linker, necessary for WebAssembly linking

#### Installation

1. **Install Xcode Command Line Tools** (if not already installed):
   ```bash
   xcode-select --install
   ```
2. **Install [Homebrew](https://brew.sh)** (if not already installed):
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```
3. **Install gcc, llvm and lld**:
   ```bash
   brew install gcc llvm lld
   ```
4. **Add gcc, llvm and lld to your PATH** (replace 14 by the installed version):
   ```bash
   echo 'export PATH="/opt/homebrew/opt/llvm/bin:$PATH"' >> ~/.zshrc
   echo 'export CC="$(brew --prefix gcc@14)/bin/gcc-14"' >> ~/.zshrc
   echo 'export CXX="$(brew --prefix gcc@14)/bin/g++-14"' >> ~/.zshrc
   echo 'alias gcc="gcc-14"' >> ~/.zshrc
   echo 'alias cc="gcc-14"' >> ~/.zshrc
   echo 'alias g++="g++-14"' >> ~/.zshrc
   echo 'alias c++="c++-14"' >> ~/.zshrc
   source ~/.zshrc
   ```
### Use wasm-pack to build WebAssembly package

After installing prerequisite, install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and build wasm crate:

   ```bash
   cargo build
   cd crates/wasm
   wasm-pack build --out-name index
   ```
## License

The code and data files in this distribution are licensed under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See <https://www.gnu.org/licenses/> for a copy of this license.

See [LICENSE](LICENSE) file
