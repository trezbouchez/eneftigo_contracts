use serde_json::json;
use workspaces::prelude::*;

const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";
const NFT_MAKE_COLLECTION_STORAGE: u128 = 79;
const STORAGE_BYTE_COST: u128 = 10_000_000_000_000_000_000;
const JS_MAX_INTEGER: u64 = 9007199254740991;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let marketplace_account = worker.dev_create_account().await?;

    let nft_contract = worker.dev_deploy(wasm).await?;

    let initialize_nft_contract = nft_contract
        .call(&worker, "new_default_meta")
        .args_json(json!({
            "owner_id": marketplace_account.id()
        }))?
        .transact()
        .await;
    println!("new_default_meta: {:?}", initialize_nft_contract);
    assert!(
        initialize_nft_contract.is_ok(),
        "new_default_meta call failed"
    );

    let make_collection_storage = NFT_MAKE_COLLECTION_STORAGE * STORAGE_BYTE_COST;
    let make_collection_outcome = marketplace_account
    .call(&worker, nft_contract.id().clone(), "make_collection")
        .args_json(json!({
            "collection_id": 0,
            "max_supply": 0
        }))?
        .deposit(make_collection_storage)
        .transact()
        .await;
    println!("make_collection: {:?}", make_collection_outcome);
    assert!(
        make_collection_outcome.is_ok(),
        "make_collection call failed"
    );
    assert!(
        make_collection_outcome
            .unwrap()
            .status
            .as_success()
            .is_some(),
        "NFT make_collection returned error"
    );

    Ok(())
}
