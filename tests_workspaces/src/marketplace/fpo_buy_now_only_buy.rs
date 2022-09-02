use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::types::Balance;
use crate::gas_and_storage::*;

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

    // Place offerring
    let title = "Bored Aardvark";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF";
    let fpo_add_worst_case_storage_cost =
        FPO_ADD_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    println!(
        "{}: Adding listing for 2 items priced 1000yN",
        "fpo_add_buy_now_only".purple()
    );
    let outcome = seller_account
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 2,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(FPO_BUY_NOW_ONLY_ADD_GAS)
        .transact()
        .await?;
    let collection_id = outcome.json::<u64>()?;

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

    /*
    CASE #01: Deposit won't cover the price
    */
    println!(
        "{}: Deposit won't cover the price:",
        "#01 fpo_buy".cyan()
    );
    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_BUY_NOW_ONLY_BUY_GAS)
        .deposit(900)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Succeeded even though it should have panicked!"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #02: Deposit sufficient to pay the price but won't cover NFT storage
    */
    println!(
        "{}: Deposit sufficient to pay the price but won't cover NFT storage:",
        "#02 fpo_buy".cyan()
    );
    let state_before = get_state(&worker, &parties).await;
    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_BUY_NOW_ONLY_BUY_GAS)
        .deposit(1000)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Succeeded even though it should have failed!"
    );

    // check if everything has been rolled back
    let state_after = get_state(&worker, &parties).await;
    assert!(
        state_before.marketplace.storage_usage == state_after.marketplace.storage_usage
            && state_before.seller.storage_usage == state_after.seller.storage_usage
            && state_before.nft.storage_usage == state_after.nft.storage_usage
            && state_before.buyer1.storage_usage == state_after.buyer1.storage_usage,
        "Storages have changed even though purchase failed!"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #03: Deposit sufficient to succeed:
    */
    println!(
        "{}: Deposit sufficient to succeed:",
        "#03 fpo_buy".cyan()
    );
    let state_before = get_state(&worker, &parties).await;

    let nft_mint_worst_case_storage_cost =
        NFT_MINT_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_BUY_NOW_ONLY_BUY_GAS)
        .deposit(1000 + nft_mint_worst_case_storage_cost)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;
    verify_balances(&outcome, &state_before, &state_after, 1000, 1);

    println!(" - {}", "PASSED".green());

    /*
    CASE #04: Buying the last available item
    */
    println!(
        "{}: Buying the last available item:",
        "#04 fpo_buy".cyan()
    );
    let state_before = state_after;

    let nft_mint_worst_case_storage_cost =
        NFT_MINT_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let outcome = buyer2_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_BUY_NOW_ONLY_BUY_GAS)
        .deposit(2000 + nft_mint_worst_case_storage_cost)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;
    verify_balances(&outcome, &state_before, &state_after, 1000, 2);

    println!(" - {}", "PASSED".green());

    /*
    CASE #05: Attempt to buy when no supply left
    */
    println!(
        "{}: Attempt to buy when no supply left:",
        "#05 fpo_buy".cyan()
    );

    let nft_mint_worst_case_storage_cost =
        NFT_MINT_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let outcome = buyer3_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_BUY_NOW_ONLY_BUY_GAS)
        .deposit(1000 + nft_mint_worst_case_storage_cost)
        .transact()
        .await;

    assert!(outcome.is_err(), "Succeeded even though no supply was left!");

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
