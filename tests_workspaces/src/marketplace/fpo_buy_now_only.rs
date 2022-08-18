use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use std::convert::TryInto;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{types::Balance, AccountId};

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

const STORAGE_COST_YOCTO_PER_BYTE: u128 = 10000000000000000000;
const NFT_MAKE_COLLECTION_STORAGE_BYTES: u128 = 79;
const CONTRACT_GAS_REWARD_RATE: u64 = 30;

#[derive(Debug)]
struct State {
    seller: workspaces::AccountDetails,
    marketplace: workspaces::AccountDetails,
    nft: workspaces::AccountDetails,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

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
    CASE #01: Deposit won't cover marketplace storage
    */
    /*    print!(
        "{}: Deposit won't cover marketplace storage",
        "CASE #01".cyan()
    );

    let seller_info = seller.view_account(&worker).await?;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let nft_info = nft_account.view_account(&worker).await?;
    let state_before = State {
        seller: seller_info,
        marketplace: marketplace_info,
        nft: nft_info,
    };

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

    println!(" - {}", "PASSED".green());*/

    /*
    CASE #02: All offering parameters correct
    */
    println!("{}: All offering parameters correct", "CASE #02".cyan());

    let seller_info = seller.view_account(&worker).await?;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let nft_info = nft_account.view_account(&worker).await?;
    let state_before = State {
        seller: seller_info,
        marketplace: marketplace_info,
        nft: nft_info,
    };

    let asset_url = "http://eneftigo/asset.png";
    let (marketplace_estimated_storage, nft_estimated_storage) =
        estimated_storage(seller.id(), nft_account.id(), asset_url);
    let total_estimated_storage_cost = (marketplace_estimated_storage + nft_estimated_storage)
        as Balance
        * STORAGE_COST_YOCTO_PER_BYTE;

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

    println!("{:#?}", outcome);

    /*    verify_balances_success(
        outcome,
        marketplace_contract.id(),
        state_before,
        state_after,
    );

    println!(" - {}", "PASSED".green());*/

    // Add new listing, attached deposit is spot-on, no refund needed
    /*    let outcome = seller
        .call(
            &worker,
            &marketplace_contract.id(),
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
        outcome.is_success(),
        "Adding FPO failed: {:#?} {}",
        outcome,
        "FAILED".red()
    );
    println!("Another buy-now-only Fixed Price Offering added successfully");*/

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
    // println!("{}", "PASSED".green());

    Ok(())
}

fn estimated_storage(seller_id: &str, nft_id: &str, asset_url: &str) -> (u64, u64) {
    let seller_id_len: u64 = seller_id.len().try_into().unwrap();
    let nft_id_len: u64 = nft_id.len().try_into().unwrap();
    let asset_url_len: u64 = asset_url.len().try_into().unwrap();
    let marketplace_storage: u64 = 670 + 2 * seller_id_len + 5 * nft_id_len;
    let nft_storage: u64 = 136 + 2 * asset_url_len;
    (marketplace_storage, nft_storage)
}

fn verify_balances_failure(state_before: State, state_after: State) {}

fn verify_balances_success(
    execution_details: CallExecutionDetails,
    marketplace_id: &workspaces::AccountId,
    state_before: State,
    state_after: State,
) {
    // https://docs.near.org/concepts/basics/transactions/gas#:~:text=**How-,much,-of%20the%20gas

    // println!("{:#?}", execution_details);

    let transaction = execution_details.outcome();
    let receipts = execution_details.receipt_outcomes();

    // static execution gas (for converting transaction into receipt)
    let static_execution_gas = transaction.gas_burnt;
    let static_execution_gas_cost = transaction.tokens_burnt;
    // println!("static execution gas {}", static_execution_gas);

    // total exeuction gas (and its cost) consumed by the contract execution
    let execution_receipt = receipts
        .iter()
        .find(|&receipt| receipt.executor_id == *marketplace_id)
        .expect("Could not locate call execution receipt");
    let dynamic_execution_gas = execution_receipt.gas_burnt;
    let dynamic_execution_gas_cost = execution_receipt.tokens_burnt;

    // here we assume that the contract gas reward will be calculated using this gas price
    let execution_gas_price = dynamic_execution_gas_cost / u128::from(dynamic_execution_gas);

    // println!("total execution gas {}", total_execution_gas);

    // calculate contract gas reward
    let marketplace_gas_reward_base = dynamic_execution_gas - static_execution_gas;
    let marketplace_gas_reward = marketplace_gas_reward_base * CONTRACT_GAS_REWARD_RATE / 100;
    let marketplace_reward = execution_gas_price * u128::from(marketplace_gas_reward);

    // println!("Contract reward {}", marketplace_reward);

    // verify marketplace balance
    let marketplace_storage_usage: u128 =
        (state_after.marketplace.storage_usage - state_before.marketplace.storage_usage).into();
    let marketplace_storage_cost = marketplace_storage_usage * STORAGE_COST_YOCTO_PER_BYTE;
    let marketplace_balance_correct =
        state_before.marketplace.balance + marketplace_storage_cost + marketplace_reward;
    assert_eq!(
        marketplace_balance_correct, state_after.marketplace.balance,
        "Marketplace balance {} is wrong. It should be {}",
        state_after.marketplace.balance, marketplace_balance_correct
    );

    // verify seller balance
    let seller_gas_cost = receipts
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + transaction.tokens_burnt;
    let seller_balance_correct =
        state_before.seller.balance - seller_gas_cost - marketplace_storage_cost;
    assert_eq!(
        state_after.seller.balance, seller_balance_correct,
        "Seller balance {} is wrong. Should be {}.",
        state_after.seller.balance, seller_balance_correct
    );
}
