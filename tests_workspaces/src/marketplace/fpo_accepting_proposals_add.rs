use crate::gas_and_storage::*;
use chrono::{TimeZone, Utc};
use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::types::Balance;

#[allow(dead_code)]
mod gas_and_storage;

#[derive(Debug)]
struct State {
    seller: workspaces::AccountDetails,
    marketplace: workspaces::AccountDetails,
    nft: workspaces::AccountDetails,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::testnet().await?;

    let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace_contract: workspaces::Contract = worker.dev_deploy(&marketplace_wasm).await?;
    println!(
        "MARKETPLACE accountId: {}",
        marketplace_contract.id().to_string(),
    );
    println!("    marketplace contract deployed");

    // Initialize Marketplace Contract
    let outcome = marketplace_contract
        .call(&worker, "new")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await?;
    assert!(
        outcome.is_success(),
        "    marketplace initialization failed: {:#?} {}",
        outcome,
        "FAILED".red()
    );
    println!("    marketplace contract initialized");

    // Create NFT subaccount
    let outcome = marketplace_contract
        .as_account()
        .create_subaccount(&worker, "nft")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(
        outcome.details.is_success(),
        "NFT subaccont creation failed: {:#?} {}",
        outcome.details,
        "FAILED".red()
    );
    let nft_account: workspaces::Account = outcome.result;
    println!("NFT accountId: {}", nft_account.id().to_string(),);

    // Deploy NFT Contract
    let nft_wasm = std::fs::read(&NFT_WASM_FILEPATH)?;
    let outcome = nft_account.deploy(&worker, &nft_wasm).await?;
    assert!(
        outcome.details.is_success(),
        "    nft contract deployment failed: {:#?} {}",
        outcome.details,
        "FAILED".red()
    );
    let nft_contract: workspaces::Contract = outcome.result;
    println!("    nft contract deployed");

    // Initialize NFT Contract
    let outcome = nft_account
        .call(&worker, &nft_contract.id(), "new_default_meta")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await?;
    assert!(
        outcome.is_success(),
        "    nft contract initialization failed {:?} {}",
        outcome,
        "FAILED".red()
    );
    println!("    nft contract initialized");

    // Create seller account
    let seller: workspaces::Account = worker.dev_create_account().await?;
    println!("SELLER accountId: {}", seller.id());

    let fpo_add_worst_case_storage_cost =
        FPO_ADD_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let now_timestamp = Utc::now().timestamp();
    let end_too_soon_timestamp = now_timestamp + MIN_DURATION_SECS / 2;
    let end_too_soon = Utc.timestamp(end_too_soon_timestamp, 0).to_rfc3339();
    let end_too_late_timestamp = now_timestamp + MAX_DURATION_SECS * 2;
    let end_too_late = Utc.timestamp(end_too_late_timestamp, 0).to_rfc3339();
    let end_valid_timestamp = now_timestamp + MAX_DURATION_SECS / 2;
    let end_valid = Utc.timestamp(end_valid_timestamp, 0).to_rfc3339();

    /*
    CASE #01: End date missing
    */
    println!(
        "{}: End date missing:",
        "fpo_add_accepting_proposals case #01".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF";

    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await;

    assert!(outcome.is_err(), "Listed despite missing end date");

    println!(" - {}", "PASSED".green());

    /*
    CASE #02: Insufficient deposit
    */
    println!(
        "{}: Insufficient deposit:",
        "fpo_add_accepting_proposals case #02".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF";

    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_valid,
        }))?
        .deposit(100)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await;

    assert!(outcome.is_err(), "Listed despite insufficient deposit");

    println!(" - {}", "PASSED".green());

    /*
    CASE #03: Duration too short
    */
    println!(
        "{}: Duration too short:",
        "fpo_add_accepting_proposals case #03".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbza";

    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_too_soon,
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await;
    assert!(outcome.is_err(), "Accepted despite too short a duration");

    println!(" - {}", "PASSED".green());

    /*
    CASE #04: Duration too long
    */
    println!(
        "{}: Duration too long:",
        "fpo_add_accepting_proposals case #04".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbza";

    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_too_late,
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await;
    assert!(outcome.is_err(), "Accepted despite too long a duration");

    println!(" - {}", "PASSED".green());

    /*
    CASE #05: All parameters correct, storage deposit sufficent
    */
    println!(
        "{}: All parameters correct, storage deposit sufficent:",
        "fpo_add_accepting_proposals case #05".cyan()
    );

    let seller_info = seller.view_account(&worker).await?;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let nft_info = nft_account.view_account(&worker).await?;
    let state_before = State {
        seller: seller_info,
        marketplace: marketplace_info,
        nft: nft_info,
    };

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbza";

    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_valid,
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await?;

    let seller_info = seller.view_account(&worker).await?;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let nft_info = nft_account.view_account(&worker).await?;
    let state_after = State {
        seller: seller_info,
        marketplace: marketplace_info,
        nft: nft_info,
    };

    verify_signer_balance(&outcome, &state_before, &state_after);

    println!(" - {}", "PASSED".green());

    /*
    CASE #06: Attempt to add listing for an already-used media URL
    */
    println!(
        "{}: Attempt to add listing for an already-used media URL:",
        "fpo_add_accepting_proposals case #06".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbza";

    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_valid,
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Accepted despite using an already-used media URL"
    );

    println!(" - {}", "PASSED".green());

    Ok(())
}

fn verify_signer_balance(
    execution_details: &CallExecutionDetails,
    state_before: &State,
    state_after: &State,
) {
    let transaction = execution_details.outcome();
    let receipts = execution_details.receipt_outcomes();

    // storage
    let marketplace_storage_usage =
        state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    let marketplace_storage_cost =
        marketplace_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let nft_storage_usage = state_after.nft.storage_usage - state_before.nft.storage_usage;
    let nft_storage_cost = nft_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    // gas
    let seller_gas_cost = receipts
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + transaction.tokens_burnt;

    // overall
    let seller_balance_correct =
        state_before.seller.balance - seller_gas_cost - marketplace_storage_cost - nft_storage_cost;
    assert_eq!(
        state_after.seller.balance, seller_balance_correct,
        "Seller balance {} is wrong. Should be {}.",
        state_after.seller.balance, seller_balance_correct
    );
}
