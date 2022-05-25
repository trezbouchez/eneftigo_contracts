use serde_json::json;
use workspaces::prelude::*;
use near_units::{parse_gas};

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace = worker.dev_deploy(wasm).await?;

    let init_outcome = marketplace
    .call(&worker, "new")
    .args_json(json!({
        "owner_id": marketplace.id(),
    }))?
    .transact()
    .await?;
    println!("new: {:?}", init_outcome);

    let add_fpo_outcome = marketplace
        .call(&worker, "fpo_add_buy_now_only")
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .gas(parse_gas!("999 Tgas") as u64)
        .transact()
        .await?;
    println!("fpo_add_buy_now_only: {:?}", add_fpo_outcome);

    Ok(())
}
