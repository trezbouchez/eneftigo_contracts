use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use std::convert::TryInto;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::types::Balance;

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

const STORAGE_COST_YOCTO_PER_BYTE: u128 = 10000000000000000000;

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
        .initial_balance(parse_near!("5 N")) // about 3.5N required for wasm storage, deducted from signer (marketplace) account
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

    // Place offerring
    let asset_url = "http://eneftigo/asset.png";
    let worst_case_storage_usage = fpo_add_worst_case_storage_usage(
        asset_url,
        seller_account.id(),
        nft_account.id(),
        None,
        None,
    );
    let worst_case_storage_cost = worst_case_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let outcome = seller_account
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "asset_url": asset_url,
            "supply_total": 2,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(worst_case_storage_cost)
        .gas(50_000_000_000_000)
        .transact()
        .await;
    println!("OUTCOME {:#?}", outcome);
    assert!(false, "FFF");
    let collection_id = outcome?.json::<u64>()?;
    println!(
        "FPO added successfully. NFT collection id: {}",
        collection_id
    );

    /*
    CASE #01: Deposit won't cover the price
    */
    println!(
        "{}: Deposit won't cover the price:",
        "fpo_buy case #01".cyan()
    );
    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(100_000_000_000_000)
        .deposit(900)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Succeeded even though it should have panicked!"
    );
    println!(" - {}", "PASSED".green());

    /*
    CASE #02:  Deposit sufficient to pay the price but won't cover NFT storage
    */
    println!(
        "{}:  Deposit sufficient to pay the price but won't cover NFT storage:",
        "fpo_buy case #02".cyan()
    );
    let state_before = get_state(&worker, &parties).await;
    let outcome = buyer1_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(100_000_000_000_000)
        .deposit(1000)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Succeeded even though it should have failed!"
    );
    let state_after = get_state(&worker, &parties).await;
    assert!(
        state_before.marketplace.storage_usage == state_after.marketplace.storage_usage
            && state_before.seller.storage_usage == state_after.seller.storage_usage
            && state_before.nft.storage_usage == state_after.nft.storage_usage
            && state_before.buyer1.storage_usage == state_after.buyer1.storage_usage,
        "Storages have changed even though purchase failed!"
    );
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

fn fpo_add_worst_case_storage_usage(
    asset_url: &str,
    seller_id: &str,
    nft_id: &str,
    start_date: Option<String>,
    end_date: Option<String>,
) -> u64 {
    fpo_add_worst_case_marketplace_storage_usage(seller_id, nft_id, start_date, end_date)
        + fpo_add_worst_case_nft_storage_usage(asset_url)
}

fn fpo_add_worst_case_marketplace_storage_usage(
    seller_id: &str,
    nft_id: &str,
    start_date: Option<String>,
    end_date: Option<String>,
) -> u64 {
    let seller_id_len: u64 = seller_id.len().try_into().unwrap();
    let nft_id_len: u64 = nft_id.len().try_into().unwrap();
    670 + 2 * seller_id_len
        + 5 * nft_id_len
        + if start_date.is_some() { 8 } else { 0 }
        + if end_date.is_some() { 8 } else { 0 }
}

fn fpo_add_worst_case_nft_storage_usage(asset_url: &str) -> u64 {
    let asset_url_len: u64 = asset_url.len().try_into().unwrap();
    136 + 2 * asset_url_len
}

fn verify_seller_balance(
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
