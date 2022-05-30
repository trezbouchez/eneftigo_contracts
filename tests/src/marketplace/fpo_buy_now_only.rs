use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();

    let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace_contract: workspaces::Contract = worker
        .dev_deploy(marketplace_wasm)
        .await
        .expect("Marketplace contract deployment failed");
    println!(
        "Marketplace contract deployed to {}",
        marketplace_contract.id().to_string()
    );

    let nft_account = marketplace_contract
        .as_account()
        .create_subaccount(&worker, "nft")
        .initial_balance(parse_near!("10 N")) // or deploy will fail
        .transact()
        .await
        .expect("Could not create NFT subaccount")
        .result;
    println!(
        "NFT contract subaccount created at {}",
        nft_account.id().to_string()
    );

    let nft_wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let nft_contract: workspaces::Contract = nft_account
        .deploy(&worker, nft_wasm)
        .await
        .expect("NFT contract deployment failed")
        .result;
    println!("NFT contract deployed to {}", nft_contract.id().to_string());

    let nft_initialize_outcome = nft_account
        .call(&worker, nft_contract.id().clone(), "new_default_meta")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await
        .expect("NFT contract initialization call failed");
    assert!(
        nft_initialize_outcome.clone().status.as_success().is_some(),
        "NFT contract initialization call returned error {:?}",
        nft_initialize_outcome.status
    );
    println!(
        "NFT contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    let marketplace_initialize_outcome = marketplace_contract
        .call(&worker, "new")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await
        .expect("Marketplace contract initialization call failed");
    assert!(
        marketplace_initialize_outcome.status.as_success().is_some(),
        "Marketplace initialization call returned error"
    );
    println!(
        "Marketplace contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    let seller = worker.dev_create_account().await?;
    let buyer = worker.dev_create_account().await?;

    let add_buy_now_fpo_status = seller
        .call(
            &worker,
            marketplace_contract.id().clone(),
            "fpo_add_buy_now_only",
        )
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(790000000000000000000)
        .gas(50_000_000_000_000)
        .transact()
        .await
        .expect("fpo_add_buy_now_only call failed")
        .status;
    println!("fpo_add_buy_now_only status: {:?}", add_buy_now_fpo_status);
    assert!(
        add_buy_now_fpo_status.as_success().is_some(),
        "fpo_add_buy_now_only call returned error"
    );

    /*    let add_proposals_fpo_outcome = seller
            .call(&worker, marketplace.id().clone(), "fpo_add_accepting_proposals")
            .args_json(json!({
                "supply_total": 10,
                "buy_now_price_yocto": "1000",
                "min_proposal_price_yocto": "500",
                "end_date": "2025-05-30T11:20:55+08:00"
            }))?
            .gas(50_000_000_000_000)
            .transact()
            .await;
        println!("fpo_add_proposals: {:?}", add_proposals_fpo_outcome);
        let add_proposals_fpo_success = add_proposals_fpo_outcome.expect("fpo_add_accepting_proposals call failed");
        // let add_proposals_fpo_outcome_json: serde_json::Value = add_proposals_fpo_success.json()?;
        // let proposals_nft_account_id_str = add_proposals_fpo_outcome_json.as_str().unwrap();
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
    */

    Ok(())
}
