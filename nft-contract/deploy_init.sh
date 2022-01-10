#!/bin/bash

near delete nft.trez.testnet trez.testnet
near create-account nft.trez.testnet --masterAccount trez.testnet --initialBalance 1
yarn build
near deploy --wasmFile ../out/main.wasm --accountId nft.trez.testnet
near call nft.trez.testnet new_default_meta '{"owner_id": "nft.trez.testnet"}' --accountId nft.trez.testnet
