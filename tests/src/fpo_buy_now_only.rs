use serde_json::json;
use workspaces::prelude::*;

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace = worker.dev_deploy(wasm).await?;
    let seller = worker.dev_create_account().await?;
    let buyer = worker.dev_create_account().await?;

    // let create_nft_subaccount_outcome = marketplace.as_account()
    // .create_subaccount(&worker, "nft")
    // .transact()
    // .add_full_access_key(env::signer_account_pk())   // TODO: what for?
    // .await;
    // assert!(create_nft_subaccount_outcome.is_ok() , "Could not create NFT subaccount");

    let marketplace_new_outcome = marketplace
        .call(&worker, "new")
        .args_json(json!({
            "owner_id": marketplace.id(),
        }))?
        .transact()
        .await;
    println!("new: {:?}", marketplace_new_outcome);
    assert!(marketplace_new_outcome.is_ok() , "Marketplace initialization call failed");
    assert!(marketplace_new_outcome.unwrap().status.as_success().is_some(), "Marketplace initialization call returned error");

    let add_buy_now_fpo_outcome = seller
        .call(&worker, marketplace.id().clone(), "fpo_add_buy_now_only")
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .gas(50_000_000_000_000)
        .transact()
        .await;
    println!("fpo_add_buy_now_only: {:?}", add_buy_now_fpo_outcome);
    let add_buy_now_fpo_success = add_buy_now_fpo_outcome.expect("fpo_add_buy_now_only call failed");
    let add_buy_now_fpo_outcome_json: serde_json::Value = add_buy_now_fpo_success.json()?;
    let buy_now_nft_account_id_str = add_buy_now_fpo_outcome_json.as_str().unwrap();
    assert!(add_buy_now_fpo_success.status.as_success().is_some(), "fpo_add_buy_now_only call returned error");

    let add_proposals_fpo_outcome = seller
        .call(&worker, marketplace.id().clone(), "fpo_add_accepting_proposals")
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": "2022-05-30T11:20:55+08:00"
        }))?
        .gas(100_000_000_000_000)
        .transact()
        .await;
    println!("fpo_add_proposals: {:?}", add_proposals_fpo_outcome);
    let add_proposals_fpo_success = add_proposals_fpo_outcome.expect("fpo_add_accepting_proposals call failed");
    let add_proposals_fpo_outcome_json: serde_json::Value = add_proposals_fpo_success.json()?;
    let proposals_nft_account_id_str = add_proposals_fpo_outcome_json.as_str().unwrap();
    assert!(add_proposals_fpo_success.status.as_success().is_some(), "fpo_add_accepting_proposals call returned error");
    
    let fpo_buy_outcome = buyer
        .call(&worker, marketplace.id().clone(), "fpo_buy")
        .args_json(json!({ "nft_contract_id": buy_now_nft_account_id_str }))?
        .gas(100_000_000_000_000)
        .deposit(1000)
        .transact()
        .await;
    println!("fpo_buy: {:?}", fpo_buy_outcome);
    let fpo_buy_success = fpo_buy_outcome.expect("fpo_buy call failed");
    assert!(fpo_buy_success.status.as_success().is_some(), "fpo_buy call returned error");

    let fpo_place_proposal_outcome = buyer
        .call(&worker, marketplace.id().clone(), "fpo_place_proposal")
        .args_json(json!({
            "nft_account_id": proposals_nft_account_id_str,
            "price_yocto": "900",
        }))?
        .gas(100_000_000_000_000)
        .deposit(1000)
        .transact()
        .await;
    println!("fpo_place_proposal: {:?}", fpo_place_proposal_outcome);
    let fpo_place_proposal_success = fpo_place_proposal_outcome.expect("fpo_place_proposal call failed");
    assert!(fpo_place_proposal_success.status.as_success().is_some(), "fpo_place_proposal call returned error");
    // let place_proposal_outcome_json: serde_json::Value = place_proposal_outcome.json()?;
    // let proposal_id = place_proposal_outcome_json.as_u64().unwrap();

    let fpo_accept_proposal_outcome = seller
        .call(&worker, marketplace.id().clone(), "fpo_accept_proposals")
        .args_json(json!({
            "nft_contract_id": proposals_nft_account_id_str,
            "accepted_proposals_count": 1,
        }))?
        .gas(100_000_000_000_000)
        .transact()
        .await;
    println!("fpo_accept_proposals: {:?}", fpo_accept_proposal_outcome);
    let fpo_accept_proposal_success = fpo_accept_proposal_outcome.expect("fpo_accept_proposals call failed");
    assert!(fpo_accept_proposal_success.status.as_success().is_some(), "fpo_accept_proposals call returned error");

    Ok(())
}
