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
    let seller: workspaces::Account = worker.dev_create_account().await?;
    println!("SELLER accountId: {}", seller.id());

    /*
    CASE #01: Deposit won't cover marketplace storage.
    */
    println!(
        "{}: Deposit won't cover marketplace storage:",
        "fpo_add_buy_now_only case #01".cyan()
    );

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "asset_url": "http://eneftigo/asset.png",
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        // .deposit(8_000_000_000_000_000_000_000)
        .gas(10_000_000_000_000)
        .transact()
        .await;

    assert!(
        outcome.is_err(),
        "Transaction succeeded despite insufficient deposit"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #02: Deposit will only cover Marketplace storage, not Nft.
    */
    println!(
        "{}: Deposit will only cover Marketplace storage, not Nft:",
        "fpo_add_buy_now_only case #02".cyan()
    );

    let asset_url = "http://eneftigo/asset0.png";
    let worst_case_marketplace_storage_usage =
        fpo_add_worst_case_marketplace_storage_usage(seller.id(), nft_account.id(), None, None);
    let worst_case_marketplace_storage_cost =
        worst_case_marketplace_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "asset_url": asset_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(worst_case_marketplace_storage_cost)
        .gas(50_000_000_000_000)
        .transact()
        .await;

    assert!(
        outcome.is_err(),
        "Succeeded despite deposit insufficient to cover NFT storage"
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #03: All offering parameters correct, storage deposit spot-on.
    */
    println!(
        "{}: All offering parameters correct, storage deposit spot-on:",
        "fpo_add_buy_now_only case #03".cyan()
    );

    let seller_info = seller.view_account(&worker).await?;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let nft_info = nft_account.view_account(&worker).await?;
    let state_before = State {
        seller: seller_info,
        marketplace: marketplace_info,
        nft: nft_info,
    };

    let asset_url = "http://eneftigo/asset1.png";
    let worst_case_storage_usage =
        fpo_add_worst_case_storage_usage(asset_url, seller.id(), nft_account.id(), None, None);
    let worst_case_storage_cost = worst_case_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "asset_url": asset_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(worst_case_storage_cost)
        .gas(50_000_000_000_000)
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
    // Verify our worst case storage computation is valid. This is not really a showstopper
    // (as long as we attach sufficient deposit) but we want to ensure the computation is ok
    // and all balances are ok in the case no refund is needed at all
    // assert_eq!(
    //     state_after.marketplace.storage_usage - state_before.marketplace.storage_usage
    //         + state_after.nft.storage_usage
    //         - state_before.nft.storage_usage,
    //     worst_case_storage_usage,
    //     "Worst case storage usage computation is incorrect!"
    // );

    verify_signer_balance(&outcome, &state_before, &state_after);
    println!(" - {}", "PASSED".green());

    /*
    CASE #04: All offering parameters correct, excess storage deposit
    */
    println!(
        "{}: All offering parameters correct, excess storage deposit:",
        "fpo_add_buy_now_only case #04".cyan()
    );
    let state_before = state_after;
    let asset_url = "http://eneftigo/asset2.png";
    let worst_case_storage_usage =
        fpo_add_worst_case_storage_usage(asset_url, seller.id(), nft_account.id(), None, None);
    let excess_storage: u64 = 100;
    let total_estimated_storage_cost =
        (worst_case_storage_usage + excess_storage) as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "asset_url": asset_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(total_estimated_storage_cost)
        .gas(50_000_000_000_000)
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
    CASE #05:Attempt to place offer for an already-existing asset causing NFT make_collection panic.
    */
    println!(
        "{}: Attempt to add offering for an already-existing asset causing NFT make_collection panic:",
        "fpo_add_buy_now_only case #05".cyan()
    );

    let state_before = state_after;

    let asset_url = "http://eneftigo/asset2.png";
    let worst_case_storage_usage =
        fpo_add_worst_case_storage_usage(asset_url, seller.id(), nft_account.id(), None, None);
    let total_estimated_storage_cost =
        worst_case_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "asset_url": asset_url,
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(total_estimated_storage_cost)
        .gas(50_000_000_000_000)
        .transact()
        .await;

    assert!(
        outcome.is_err(),
        "Succeeded despite NFT asset URL collission"
    );

    println!(" - {}", "PASSED".green());

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
    // println!("{}", "PASSED".green());

    Ok(())
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
