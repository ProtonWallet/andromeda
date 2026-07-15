# Base CI image for andromeda.
# Bakes in the slow-to-build Rust tooling so pipeline jobs don't recompile them
# on every run. Pin the tag to the Rust version, e.g. andromeda-ci:rust-1.96.0.
FROM harbor.protontech.ch/docker.io/library/rust:1.96.0

# Rustup components used by the format/lint jobs.
RUN rustup component add rustfmt clippy

# Cargo tools compiled from source (the slow ones).
RUN cargo install cargo2junit \
                  cargo-tarpaulin \
                  wasm-pack \
                  cargo-auditable \
                  cargo-audit

# System packages for the wasm build.
RUN apt-get update \
  && apt-get install -y clang \
  && rm -rf /var/lib/apt/lists/*

# Binaryen (provides wasm-opt) for the wasm build.
RUN wget -qO- https://github.com/WebAssembly/binaryen/releases/download/version_117/binaryen-version_117-x86_64-linux.tar.gz \
      | gunzip | tar xvf - -C /tmp \
  && mv /tmp/binaryen-version_117/bin/* /bin/ \
  && mv /tmp/binaryen-version_117/lib/* /lib/ \
  && rm -rf /tmp/binaryen-version_117
