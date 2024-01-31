# The Bitcoin Dev Kit

<div align="center">
  <h1>Andromeda</h1>

  <img src="https://res.cloudinary.com/dbulfrlrz/image/upload/v1693233221/static/logos/proton-logo_z7innb.svg" width="220" />

  <p>
    <strong>Proton's toolsuit to integrate Bitcoin and Lightning into its ecosystem</strong>
  </p>

  <h4>
    <a href="https://proton.me/">Proton Home</a>
    <span> | </span>
    <a href="https://docs.rs/andromeda">Documentation</a>
  </h4>
</div>

## About

The `andromeda` libraries aims to provide logical blocks to build a privacy-focused, cross-platform, self-custody Bitcoin and Lightning wallet, integrated in
Proton's ecosystem

## Architecture

The project is split up into several crates in the `/crates` directory:

- [`api`](./crates/api): Contains an api client to call Proton Wallet backend HTTP API
- [`vss`](./crates/vss): (TBD) A Versioned Storage Service client, used to persist encrypted chain data and lightning channel state
- [`bitcoin`](./crates/bitcoin): A library that provides utilities to use bitcoin on the 1rst layer such as chain syncing, transactions/balance/utxos retrieving, address generating and obviously transaction building, signing and broadcasting. 
- [`coinjoin`](./crates/coinjoin): (TBD) A rust client for Whirlpool coinjoin protocol
- [`lightning`](./crates/lightning): (TBD) A LDK-based lightning node that support BOLT11 invoice generation and payment
- [`key-transparency`](./crates/key-transparency): (TBD) A rust client for Proton's Key Transparency protocol
- [`wasm`](./crates/wasm): Relevant interfaces to WASM

## External dependencies

[`bdk`]: https://docs.rs/bdk/
[`pdk`]: https://docs.rs/pdk/
[`ldk`]: https://docs.rs/ldk/
[`muon`]: https://docs.rs/muon/

## License

The code and data files in this distribution are licensed under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See <https://www.gnu.org/licenses/> for a copy of this license.

See [LICENSE](LICENSE) file
