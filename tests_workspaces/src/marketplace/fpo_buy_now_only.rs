use colored::Colorize;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{types::Balance, AccountId};

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

const STORAGE_COST_YOCTO_PER_BYTE: u128 = 10000000000000000000;
const NFT_MAKE_COLLECTION_STORAGE_BYTES: u128 = 79;
const CONTRACT_GAS_REWARD_RATE: u64 = 30;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::testnet().await?;

    let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace_contract: workspaces::Contract = worker.dev_deploy(&marketplace_wasm).await?;
    println!(
        "Marketplace contract deployed to {}",
        marketplace_contract.id().to_string(),
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
        outcome.is_success(),
        "Marketplace initialization failed: {:#?} {}",
        outcome,
        "FAILED".red()
    );
    println!(
        "Marketplace contract initialized with owner {}",
        marketplace_contract.id().to_string(),
    );

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
    println!(
        "NFT contract subaccount created at {}",
        nft_account.id().to_string(),
    );

    // Deploy NFT Contract
    let nft_wasm = std::fs::read(&NFT_WASM_FILEPATH)?;
    let outcome = nft_account.deploy(&worker, &nft_wasm).await?;
    assert!(
        outcome.details.is_success(),
        "NFT contract deployment failed: {:#?} {}",
        outcome.details,
        "FAILED".red()
    );
    let nft_contract: workspaces::Contract = outcome.result;
    println!("NFT contract deployed to {}", nft_contract.id().to_string());

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
        "NFT contract initialization failed {:?} {}",
        outcome,
        "FAILED".red()
    );
    println!(
        "NFT contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    // Create seller account
    let seller: workspaces::Account = worker.dev_create_account().await?;
    println!("Seller account created at {}", seller.id());

    // Calculate storage usage
    let estimated_marketplace_storage_usage = 670
        + 2 * seller.id().clone().to_string().len()
        + 5 * nft_contract.id().clone().to_string().len();
    // + if start_date.is_some() { 8 } else { 0 }
    // + if end_date.is_some() { 8 } else { 0 };
    let estimated_total_storage_cost = (estimated_marketplace_storage_usage as u128
        + NFT_MAKE_COLLECTION_STORAGE_BYTES)
        * STORAGE_COST_YOCTO_PER_BYTE;

    /*
    CASE #01: Deposit won't cover marketplace storage
    */
    print!(
        "{}: Deposit won't cover marketplace storage",
        "CASE #01".cyan()
    );

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(8_000_000_000_000_000_000_000)
        .gas(10_000_000_000_000)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Adding FPO succeeded despite insifficient deposit. {}",
        "FAILED".red()
    );

    println!(" - {}", "PASSED".green());

    /*
    CASE #02: All offering parameters correct
    */
    print!("{}: All offering parameters correct", "CASE #02".cyan());

    let seller_info = seller.view_account(&worker).await?;
    let seller_balance_0 = seller_info.balance;
    let seller_storage_0 = seller_info.storage_usage;
    let nft_info = nft_account.view_account(&worker).await?;
    let nft_balance_0 = nft_info.balance;
    let nft_storage_0 = nft_info.storage_usage;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let marketplace_balance_0 = marketplace_info.balance;
    let marketplace_storage_0 = marketplace_info.storage_usage;

    let outcome = seller
        .call(&worker, &marketplace_contract.id(), "fpo_add_buy_now_only")
        .args_json(json!({
            "supply_total": 10,
            "buy_now_price_yocto": "1000",
        }))?
        .deposit(10_000_000_000_000_000_000_000)
        .gas(10_000_000_000_000)
        .transact()
        .await?;
    assert!(
        outcome.is_success(),
        "Adding FPO failed: {:#?} {}",
        outcome,
        "FAILED".red()
    );

    let seller_info = seller.view_account(&worker).await?;
    let seller_balance_1 = seller_info.balance;
    let seller_storage_1 = seller_info.storage_usage;
    let nft_info = nft_account.view_account(&worker).await?;
    let nft_balance_1 = nft_info.balance;
    let nft_storage_1 = nft_info.storage_usage;
    let marketplace_info = marketplace_contract.view_account(&worker).await?;
    let marketplace_balance_1 = marketplace_info.balance;
    let marketplace_storage_1 = marketplace_info.storage_usage;

    verify_add_outcome(
        &outcome,
        marketplace_contract.id(),
        marketplace_storage_0,
        marketplace_storage_1,
        marketplace_balance_0,
        marketplace_balance_1,
        seller_balance_0,
        seller_balance_1,
    );

    println!(" - {}", "PASSED".green());

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

fn gas_paid_by_signer(ced: &CallExecutionDetails) -> u64 {
    ced.outcomes()
        .into_iter()
        .filter(|outcome| !outcome.receipt_ids.is_empty())
        .fold(0, |accu, paid_outcome| accu + paid_outcome.gas_burnt)
}

fn gas_cost(gas: u64) -> u128 {
    (gas * 100_000_000).into()
}

fn verify_add_outcome(
    execution_details: &CallExecutionDetails,
    marketplace_account_id: &AccountId,
    marketplace_storage_before: u64,
    marketplace_storage_after: u64,
    marketplace_balance_before: Balance,
    marketplace_balance_after: Balance,
    seller_balance_before: Balance,
    seller_balance_after: Balance,
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
        .find(|&receipt| receipt.executor_id == *marketplace_account_id)
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
        (marketplace_storage_after - marketplace_storage_before).into();
    let marketplace_storage_cost = marketplace_storage_usage * STORAGE_COST_YOCTO_PER_BYTE;
    let marketplace_balance_correct =
        marketplace_balance_before + marketplace_storage_cost + marketplace_reward;
    assert_eq!(marketplace_balance_correct, marketplace_balance_after, "Marketplace balance {} is wrong. It should be {}. The computation results from: initial_balance ({}) + marketplace_storage_cost ({}) + gas_reward ({})", marketplace_balance_after, marketplace_balance_correct, marketplace_balance_before, marketplace_storage_cost, marketplace_reward);

    // verify seller balance
    let seller_gas_cost = receipts
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + transaction.tokens_burnt;
    let seller_balance_correct = seller_balance_before - seller_gas_cost - marketplace_storage_cost;
    assert_eq!(
        seller_balance_after, seller_balance_correct,
        "Seller balance {} is wrong. Should be {}.",
        seller_balance_after, seller_balance_correct
    );
}
