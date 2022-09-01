use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use std::convert::TryInto;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::types::Balance;

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

const ACCOUNT_NAME_LEN_MAX: usize = 64;     //https://nomicon.io/DataStructures/Account
const STORAGE_COST_YOCTO_PER_BYTE: u128 = 10000000000000000000;

const FPO_ADD_WORST_CASE_MARKETPLACE_STORAGE: u64 = 1349; // actual, measured
const NEW_COLLECTION_WORST_CASE_NFT_STORAGE: u64 = 422; // actual, measured
const FPO_ADD_WORST_CASE_STORAGE = FPO_ADD_WORST_CASE_MARKETPLACE_STORAGE + NEW_COLLECTION_WORST_CASE_NFT_STORAGE;

#[derive(Debug)]
struct State {
    seller: workspaces::AccountDetails,
    marketplace: workspaces::AccountDetails,
    nft: workspaces::AccountDetails,
}

/*
The marketplace deposit-related balance flow is:

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE
markertplace:   0                           0
nft:            0                           0

> A: Marketplace::fpo_add_xxx call:

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE-SIGNER_DEPOSIT
markertplace:   0                           SIGNER_DEPOSIT
nft:            0                           0

>A1: Inserts new fpo:

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE-SIGNER_DEPOSIT
markertplace:   FPO_STORAGE_COST            SIGNER_DEPOSIT-FPO_STORAGE_COST
nft:            0                           0

> B. Nft::make_collection call, attaching remaining deposit balance of SIGNER_DEPOSIT - FPO_STORAGE_COST

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE-SIGNER_DEPOSIT
markertplace:   FPO_STORAGE_COST            0
nft:            0                           SIGNER_DEPOSIT-FPO_STORAGE_COST

> B1: Nft inserts new collection:

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE-SIGNER_DEPOSIT
markertplace:   FPO_STORAGE_COST            0
nft:            NFT_STORAGE_COST            SIGNER_DEPOSIT-FPO_STORAGE_COST-NFT_STORAGE_COST

>B2:: Nft refunds remaining deposit to Marketplace

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE-SIGNER_DEPOSIT
markertplace:   FPO_STORAGE_COST            SIGNER_DEPOSIT-FPO_STORAGE_COST-NFT_STORAGE_COST
nft:            NFT_STORAGE_COST            0

>C: Marketplace refunds remaining deposit to the signer in its callback receipt

PARTY           STORAGE_COST                BALANCE
---------------------------------------------------------------
signer:         0                           SIGNER_INIT_BALANCE-FPO_STORAGE_COST-NFT_STORAGE_COST
markertplace:   FPO_STORAGE_COST            0
nft:            NFT_STORAGE_COST            0
*/

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

    let fpo_add_worst_case_storage_cost = FPO_ADD_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    
    /*
    CASE #01: Deposit won't cover marketplace storage.
    */
    println!(
        "{}: Deposit won't cover marketplace storage:",
        "fpo_add_buy_now_only case #01".cyan()
    );

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF";

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(100)
        .gas(10_000_000_000_000)
        .transact()
        .await;

    assert!(
        outcome.is_err(),
        "Transaction succeeded despite insufficient deposit"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #02: All offering parameters correct, storage deposit sufficent
    */
    println!(
        "{}: All offering parameters correct, storage deposit sufficient:",
        "fpo_add_buy_now_only case #02".cyan()
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
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(80_000_000_000_000)
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
    CASE #03:Attempt to place offer for an already-existing asset causing NFT make_collection panic.
    */
    println!(
        "{}: Attempt to add offering for an already-existing asset causing NFT make_collection panic:",
        "fpo_add_buy_now_only case #03".cyan()
    );

    let state_before = state_after;

    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbza";

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(fpo_add_worst_case_storage_cost)
        .gas(50_000_000_000_000)
        .transact()
        .await;

    assert!(
        outcome.is_err(),
        "Succeeded despite NFT asset URL collission"
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
