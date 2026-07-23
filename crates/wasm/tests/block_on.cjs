/**
 * JS integration smoke test for ProtonWalletApiClient::from_config → block_on (wasm path).
 *
 * Uses the nodejs-target wasm build in pkg-node/ (auto-initialized on require).
 */
const { join } = require("node:path");
const { WasmProtonWalletApiClient } = require(join(__dirname, "..", "pkg-node", "index.js"));

const APP_VERSION = "web-wallet@5.0.999.999-dev";
const USER_AGENT =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

const client = new WasmProtonWalletApiClient(
  APP_VERSION,
  USER_AGENT,
  undefined,
  undefined,
  "http://127.0.0.1:9",
  undefined,
);

const clients = client.clients();
if (!clients?.wallet) {
  throw new Error("WasmProtonWalletApiClient.clients() did not return wallet client");
}

client.free();
console.log("block_on wasm JS integration test passed");
