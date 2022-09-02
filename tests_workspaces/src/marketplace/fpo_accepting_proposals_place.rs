use crate::gas_and_storage::*;
use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::types::Balance;
use chrono::{Utc,TimeZone};

#[allow(dead_code)]
mod gas_and_storage;

struct Parties<'a> {
    marketplace: &'a workspaces::Account,
    nft: &'a workspaces::Account,
    seller: &'a workspaces::Account,
    buyer1: &'a workspaces::Account,
    buyer2: &'a workspaces::Account,
    buyer3: &'a workspaces::Account,
}

#[derive(Debug)]
struct State {
    marketplace: workspaces::AccountDetails,
    nft: workspaces::AccountDetails,
    seller: workspaces::AccountDetails,
    buyer1: workspaces::AccountDetails,
    buyer2: workspaces::AccountDetails,
    buyer3: workspaces::AccountDetails,
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
        .initial_balance(parse_near!("10 N")) // about 3.5N required for wasm storage, deducted from signer (marketplace) account
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
    let seller_account = worker.dev_create_account().await?;
    println!("SELLER accountId: {}", seller_account.id());

    // Create buyer accounts
    let buyer1_account = worker.dev_create_account().await?;
    println!("BUYER #1 accountId: {}", buyer1_account.id());
    let buyer2_account = worker.dev_create_account().await?;
    println!("BUYER #2 accountId: {}", buyer2_account.id());
    let buyer3_account = worker.dev_create_account().await?;
    println!("BUYER #3 accountId: {}", buyer3_account.id());

    let parties = Parties {
        marketplace: marketplace_contract.as_account(),
        nft: &nft_account,
        seller: &seller_account,
        buyer1: &buyer1_account,
        buyer2: &buyer2_account,
        buyer3: &buyer3_account,
    };

    let fpo_add_worst_case_storage_cost =
    FPO_ADD_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let fpo_place_proposal_worst_case_storage_cost = FPO_ACCEPTING_PROPOSALS_PLACE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let now_timestamp = Utc::now().timestamp();
    let end_too_soon_timestamp = now_timestamp + MIN_DURATION_SECS / 2;
    let end_too_soon = Utc.timestamp(end_too_soon_timestamp, 0).to_rfc3339();
    let end_too_late_timestamp = now_timestamp + MAX_DURATION_SECS * 2;
    let end_too_late = Utc.timestamp(end_too_late_timestamp, 0).to_rfc3339();
    let end_valid_timestamp = now_timestamp + MAX_DURATION_SECS / 2;
    let end_valid = Utc.timestamp(end_valid_timestamp, 0).to_rfc3339();

    // Add offerring
    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF";
    let outcome = seller_account
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 2,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_valid,
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await?;
    let collection_id = outcome.json::<u64>()?;
    println!("Fixed Price Offering listing added, collection ID {}", collection_id);

    /*
    CASE #01: Proposed price too low
    */
    println!(
        "{}: Proposed price too low:",
        "fpo_place_proposal case #01".cyan()
    );
    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "400",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(400 + fpo_place_proposal_worst_case_storage_cost)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Succeeded even though proposed price is too low"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #02 Proposed price of 550yN acceptable, deposit 500yN is too low
    */
    println!(
        "{}: Proposed price of 550yN acceptable, deposit 500yN is too low:",
        "fpo_place_proposal case #02".cyan()
    );

    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "550",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(500)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Succeeded even though insufficent deposit"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #03 Proposed price of 550yN acceptable, deposit sufficient
    */
    println!(
        "{}: Proposed price of 550yN acceptable, deposit sufficient:",
        "fpo_place_proposal case #03".cyan()
    );

    let proposal1_price: Balance = 550;
    let state_before = get_state(&worker, &parties).await;

    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "550",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(proposal1_price + fpo_place_proposal_worst_case_storage_cost)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;
    let tokens_burnt = get_tokens_burnt(&outcome);
    let marketplace_storage_usage = state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    let marketplace_storage_cost = marketplace_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let proposer_balance_correct = state_before.buyer1.balance - tokens_burnt - proposal1_price - marketplace_storage_cost;
    assert_eq!(state_after.buyer1.balance, proposer_balance_correct);

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
    let buyer1_info = parties
        .buyer1
        .view_account(worker)
        .await
        .expect("Error reading account state");
    let buyer2_info = parties
        .buyer2
        .view_account(worker)
        .await
        .expect("Error reading account state");
    let buyer3_info = parties
        .buyer3
        .view_account(worker)
        .await
        .expect("Error reading account state");
    State {
        marketplace: marketplace_info,
        nft: nft_info,
        seller: seller_info,
        buyer1: buyer1_info,
        buyer2: buyer2_info,
        buyer3: buyer3_info,
    }
}

fn get_tokens_burnt(execution_details: &CallExecutionDetails) -> Balance {
    execution_details.receipt_outcomes()
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + execution_details.outcome().tokens_burnt
}

fn verify_balances(
    execution_details: &CallExecutionDetails,
    state_before: &State,
    state_after: &State,
    price_paid: Balance,
    buyer_index: usize, // 1-based
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
    let buyer_gas_cost = receipts
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + transaction.tokens_burnt;

    // overall
    let (buyer_state_before, buyer_state_after) = match buyer_index {
        1 => (&state_before.buyer1, &state_after.buyer1),
        2 => (&state_before.buyer2, &state_after.buyer2),
        3 => (&state_before.buyer3, &state_after.buyer3),
        _ => panic!("Wrong buyer index"),
    };

    // buyer balance
    let buyer_balance_correct = buyer_state_before.balance
        - buyer_gas_cost
        - marketplace_storage_cost
        - nft_storage_cost
        - price_paid;
    assert_eq!(
        buyer_state_after.balance, buyer_balance_correct,
        "Buyer{} balance of {} is wrong. Should be {}",
        buyer_index, buyer_state_after.balance, buyer_balance_correct
    );

    // seller balance
    let seller_balance_correct = state_before.seller.balance + price_paid;
    assert_eq!(
        state_after.seller.balance, seller_balance_correct,
        "Seller balance of {} is wrong. Should be {}",
        state_after.seller.balance, seller_balance_correct
    );
}
