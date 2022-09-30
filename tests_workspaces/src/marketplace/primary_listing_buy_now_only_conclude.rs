use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::types::{Balance};
use crate::gas_and_storage::*;

#[allow(dead_code)]
mod gas_and_storage;

struct Parties<'a> {
    marketplace: &'a workspaces::Account,
    nft: &'a workspaces::Account,
    seller: &'a workspaces::Account,
    buyer: &'a workspaces::Account,
}

#[derive(Debug)]
struct State {
    marketplace: workspaces::AccountDetails,
    nft: workspaces::AccountDetails,
    seller: workspaces::AccountDetails,
    buyer: workspaces::AccountDetails,
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
    let seller_account: workspaces::Account = worker.dev_create_account().await?;
    println!("SELLER accountId: {}", seller_account.id());

    // Create buyer accounts
    let buyer_account = worker.dev_create_account().await?;
    println!("BUYER accountId: {}", buyer_account.id());

    let parties = Parties {
        marketplace: marketplace_contract.as_account(),
        nft: &nft_account,
        seller: &seller_account,
        buyer: &buyer_account,
    };

    let primary_listing_add_worst_case_storage_cost =
    PRIMARY_LISTING_ADD_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    /*
    CASE #01: Conclude an open-ended listing
    */

    println!(
        "{}: Conclude an open-ended listing:",
        "#01 primary_listing_conclude".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbza";
    let outcome = seller_account
        .call(&worker, &marketplace_contract.id(), "primary_listing_add_buy_now_only")
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 1,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(primary_listing_add_worst_case_storage_cost)
        .gas(PRIMARY_LISTING_BUY_NOW_ONLY_ADD_GAS)
        .transact()
        .await?;
    let collection_id = outcome.json::<u64>()?;

    let state_before = get_state(&worker, &parties).await;
    let outcome = seller_account
        .call(&worker, &marketplace_contract.id(), "primary_listing_conclude")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(PRIMARY_LISTING_BUY_NOW_ONLY_CONCLUDE_GAS)
        .transact()
        .await?;
    let state_after = get_state(&worker, &parties).await;
    verify_balances(&outcome, &state_before, &state_after);

    println!(" - {}", "PASSED".green());

    /*
    CASE #02: Conclude a time-limited listing after all purchased
    */

    println!(
        "{}: Conclude a time-limited listing after all purchased:",
        "#02 primary_listing_conclude".cyan()
    );

    let nft_mint_worst_case_storage_cost =
        NFT_MINT_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let _outcome = buyer_account
        .call(&worker, marketplace_contract.id(), "primary_listing_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(PRIMARY_LISTING_BUY_NOW_ONLY_BUY_GAS)
        .deposit(1000 + nft_mint_worst_case_storage_cost)
        .transact()
        .await?;

    let state_before = get_state(&worker, &parties).await;

    let outcome = seller_account
        .call(&worker, &marketplace_contract.id(), "primary_listing_conclude")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(PRIMARY_LISTING_BUY_NOW_ONLY_CONCLUDE_GAS)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;
    verify_balances(&outcome, &state_before, &state_after);

    println!(" - {}", "PASSED".green());

    Ok(())
}

async fn get_state<'a, T>(worker: &workspaces::Worker<T>, parties: &'a Parties<'a>) -> State
where
    T: std::marker::Sync
        + workspaces::network::NetworkInfo
        + std::marker::Send
        + workspaces::network::NetworkClient,
{
    let marketplace_info = parties
        .marketplace
        .view_account(worker)
        .await
        .expect("Error reading account state");
    let nft_info = parties
        .nft
        .view_account(worker)
        .await
        .expect("Error reading account state");
    let seller_info = parties
        .seller
        .view_account(worker)
        .await
        .expect("Error reading account state");
    let buyer_info = parties
        .buyer
        .view_account(worker)
        .await
        .expect("Error reading account state");
    State {
        marketplace: marketplace_info,
        nft: nft_info,
        seller: seller_info,
        buyer: buyer_info,
    }
}

fn verify_balances(
    execution_details: &CallExecutionDetails,
    state_before: &State,
    state_after: &State,
) {
    let transaction = execution_details.outcome();
    let receipts = execution_details.receipt_outcomes();

    // storage
    let marketplace_storage_freed =
        state_before.marketplace.storage_usage - state_after.marketplace.storage_usage;
    let marketplace_storage_cost =
        marketplace_storage_freed as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    assert_eq!(
        state_after.nft.storage_usage,
        state_before.nft.storage_usage
    );
    assert_eq!(
        state_after.buyer.storage_usage,
        state_before.buyer.storage_usage
    );

    // gas
    let seller_gas_cost = receipts
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + transaction.tokens_burnt;

// println!("SELLER_GAS {}", seller_gas_cost);

    // seller balance
    let seller_balance_correct =
        state_before.seller.balance - seller_gas_cost + marketplace_storage_cost;
    assert_eq!(
        state_after.seller.balance, seller_balance_correct,
        "Seller balance of {} is wrong. Should be {}",
        state_after.seller.balance, seller_balance_correct
    );
}
