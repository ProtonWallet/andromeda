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

- [`bdk`](https://docs.rs/bdk/)

## License

The code and data files in this distribution are licensed under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See <https://www.gnu.org/licenses/> for a copy of this license.

See [LICENSE](LICENSE) file
