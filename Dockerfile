# Base CI image for andromeda.
# Bakes in the slow-to-build Rust tooling so pipeline jobs don't recompile them
# on every run. Pin the tag to the Rust version, e.g. andromeda-ci:rust-1.96.0.
FROM harbor.protontech.ch/docker.io/library/rust:1.96.0

# Binaryen version to install, bump accordingly with what is used with wasm-pack.
ARG BINARYEN_VERSION=130

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
  && apt-get install -y --no-install-recommends clang \
  && rm -rf /var/lib/apt/lists/*

# Binaryen (provides wasm-opt) for the wasm build.
RUN wget -qO- https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz \
      | gunzip | tar xvf - -C /tmp \
  && mv /tmp/binaryen-version_${BINARYEN_VERSION}/bin/* /bin/ \
  && mv /tmp/binaryen-version_${BINARYEN_VERSION}/lib/* /lib/ \
  && rm -rf /tmp/binaryen-version_${BINARYEN_VERSION}

RUN groupadd --gid 1000 ci \
  && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash ci \
  && chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME"

USER ci
