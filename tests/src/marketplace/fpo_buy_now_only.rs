use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";
const BALANCE_WASM_FILEPATH: &str = "../out/balance.wasm";

const STORAGE_COST_YOCTO_PER_BYTE: u128 = 10000000000000000000;
const NFT_MAKE_COLLECTION_STORAGE_BYTES: u128 = 79;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();

    let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace_contract: workspaces::Contract = worker.dev_deploy(marketplace_wasm).await?;
    println!(
        "Marketplace contract deployed to {}",
        marketplace_contract.id().to_string()
    );

    // Initialize Marketplace Contract
    let outcome = marketplace_contract
        .call(&worker, "new")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await?;
    assert!(
        outcome.status.clone().as_success().is_some(),
        "Marketplace initialization failed: {:#?} {}",
        outcome.status,
        "FAILED".red()
    );
    println!(
        "Marketplace contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    // Create NFT subaccount
    let outcome = marketplace_contract
        .as_account()
        .create_subaccount(&worker, "nft")
        .initial_balance(parse_near!("5 N")) // some 3.5N is required for storage or deploy will fail
        .transact()
        .await?;
    assert!(
        outcome.details.status.clone().as_success().is_some(),
        "NFT subaccont creation failed: {:#?} {}",
        outcome.details.status,
        "FAILED".red()
    );
    let nft_account: workspaces::Account = outcome.result;
    println!(
        "NFT contract subaccount created at {}",
        nft_account.id().to_string()
    );

    // Deploy NFT Contract
    let nft_wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let outcome = nft_account.deploy(&worker, nft_wasm).await?;
    assert!(
        outcome.details.status.clone().as_success().is_some(),
        "NFT contract deployment failed: {:#?} {}",
        outcome.details.status,
        "FAILED".red()
    );
    let nft_contract: workspaces::Contract = outcome.result;
    println!("NFT contract deployed to {}", nft_contract.id().to_string());

    // Initialize NFT Contract
    let outcome = nft_account
        .call(&worker, nft_contract.id().clone(), "new_default_meta")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await?;
    assert!(
        outcome.clone().status.as_success().is_some(),
        "NFT contract initialization failed {:?} {}",
        outcome.status,
        "FAILED".red()
    );
    println!(
        "NFT contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    // Add Fixed Price Offering
    let seller: workspaces::Account = worker.dev_create_account().await?;
    println!("Seller account created at {}", seller.id());

    // Deploy Balance Contract to seller account
    let balance_wasm = std::fs::read(BALANCE_WASM_FILEPATH)?;
    let outcome = seller.deploy(&worker, balance_wasm).await?;
    assert!(
        outcome.details.status.clone().as_success().is_some(),
        "Balance contract deployment failed: {:#?} {}",
        outcome.details.status,
        "FAILED".red()
    );
    let balance_contract: workspaces::Contract = outcome.result;
    println!("Balance contract deployed to {}", seller.id().to_string());

    // Check initial seller balance
    let balance: serde_json::Value = balance_contract.view(
            &worker,
            "balance",
            json!({})
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    println!("BALANCE: {}", balance);

    let estimated_marketplace_storage_usage = 670
        + 2 * seller.id().clone().to_string().len()
        + 5 * nft_contract.id().clone().to_string().len();
    // + if start_date.is_some() { 8 } else { 0 }
    // + if end_date.is_some() { 8 } else { 0 };
    let estimated_total_storage_cost = (estimated_marketplace_storage_usage as u128
        + NFT_MAKE_COLLECTION_STORAGE_BYTES)
        * STORAGE_COST_YOCTO_PER_BYTE;

    // Add new listing, attached deposit is spot-on, no refund needed
    // println!("Initial seller balance is {:?} yoctoNEAR", seller.balance());
    let outcome = seller
        .call(
            &worker,
            marketplace_contract.id().clone(),
            "fpo_add_buy_now_only",
        )
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(estimated_total_storage_cost)
        .gas(50_000_000_000_000)
        .transact()
        .await?;
    assert!(
        outcome.status.clone().as_success().is_some(),
        "Adding FPO failed: {:#?} {}",
        outcome.status,
        "FAILED".red()
    );
    println!("Buy-now-only Fixed Price Offering added successfully");

    // Add new listing, attached deposit is spot-on, no refund needed
    let outcome = seller
        .call(
            &worker,
            marketplace_contract.id().clone(),
            "fpo_add_buy_now_only",
        )
        .args_json(json!({
            "supply_total": 50,
            "buy_now_price_yocto": "2000",
        }))?
        .deposit(estimated_total_storage_cost)
        .gas(50_000_000_000_000)
        .transact()
        .await?;
    assert!(
        outcome.status.clone().as_success().is_some(),
        "Adding FPO failed: {:#?} {}",
        outcome.status,
        "FAILED".red()
    );
    println!("Another buy-now-only Fixed Price Offering added successfully");
    // Add FPO listing
    /*    let outcome = seller
        .call(
            &worker,
            marketplace_contract.id().clone(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": "2025-05-30T11:20:55+08:00"
        }))?
        .deposit(790000000000000000000)
        .gas(50_000_000_000_000)
        .transact()
        .await?;
    assert!(
        outcome.status.clone().as_success().is_some(),
        "fpo_add_accepting_proposals failed: {:?} {}",
        outcome,
        "FAILED".red()
    );
    println!("Proposals-accepting Fixed Price Offering added successfully");*/

    // let add_proposals_fpo_outcome_json: serde_json::Value = add_proposals_fpo_success.json()?;
    // let proposals_nft_account_id_str = add_proposals_fpo_outcome_json.as_str().unwrap();

    /*        let fpo_buy_outcome = buyer
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

    // All OK
    println!("{}", "PASSED".green());

    Ok(())
}
