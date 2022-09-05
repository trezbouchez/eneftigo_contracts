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

struct Parties<'a> {
    marketplace: &'a workspaces::Account,
    fees: &'a workspaces::Account,
    nft: &'a workspaces::Account,
    seller: &'a workspaces::Account,
    buyer1: &'a workspaces::Account,
    buyer2: &'a workspaces::Account,
    buyer3: &'a workspaces::Account,
}

#[derive(Debug)]
struct State {
    marketplace: workspaces::AccountDetails,
    fees: workspaces::AccountDetails,
    nft: workspaces::AccountDetails,
    seller: workspaces::AccountDetails,
    buyer1: workspaces::AccountDetails,
    buyer2: workspaces::AccountDetails,
    buyer3: workspaces::AccountDetails,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let worker = workspaces::testnet().await?;
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

    // Create Fees subaccount
    let outcome = marketplace_contract
        .as_account()
        .create_subaccount(&worker, "fees")
        .initial_balance(parse_near!("10 N")) // about 3.5N required for wasm storage, deducted from signer (marketplace) account
        .transact()
        .await?;
    assert!(
        outcome.details.is_success(),
        "FEES subaccont creation failed: {:#?} {}",
        outcome.details,
        "FAILED".red()
    );
    let fees_account: workspaces::Account = outcome.result;
    println!("FEES accountId: {}", fees_account.id().to_string(),);

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
        fees: &fees_account,
        nft: &nft_account,
        seller: &seller_account,
        buyer1: &buyer1_account,
        buyer2: &buyer2_account,
        buyer3: &buyer3_account,
    };

    let fpo_add_worst_case_base_storage_cost =
        FPO_ADD_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let fpo_place_proposal_worst_case_storage_cost =
        FPO_ACCEPTING_PROPOSALS_PLACE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    let now_timestamp = Utc::now().timestamp();
    let end_too_soon_timestamp = now_timestamp + MIN_DURATION_SECS / 2;
    let end_too_soon = Utc.timestamp(end_too_soon_timestamp, 0).to_rfc3339();
    let end_too_late_timestamp = now_timestamp + MAX_DURATION_SECS * 2;
    let end_too_late = Utc.timestamp(end_too_late_timestamp, 0).to_rfc3339();
    let end_valid_timestamp = now_timestamp + MAX_DURATION_SECS / 2;
    let end_valid = Utc.timestamp(end_valid_timestamp, 0).to_rfc3339();

    let state_initial = get_state(&worker, &parties).await;

    // Add offerring
    let title = "Bored Grapes";
    let media_url = "https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF";
    let total_supply = 2u64;
    let required_deposit = fpo_add_worst_case_base_storage_cost;
    let state_before = get_state(&worker, &parties).await;

    let mut seller_tokens_burnt: Balance = 0;
    let mut buyer1_tokens_burnt: Balance = 0;
    let mut buyer2_tokens_burnt: Balance = 0;
    let mut buyer3_tokens_burnt: Balance = 0;

    let outcome = seller_account
        .call(
            &worker,
            &marketplace_contract.id(),
            "fpo_add_accepting_proposals",
        )
        .args_json(json!({
            "title": title,
            "media_url": media_url,
            "supply_total": total_supply,
            "buy_now_price_yocto": "1000",
            "min_proposal_price_yocto": "500",
            "end_date": end_valid,
        }))?
        .deposit(required_deposit)
        .gas(FPO_ACCEPTING_PROPOSALS_ADD_GAS)
        .transact()
        .await?;
    let collection_id = outcome.json::<u64>()?;
    let state_after = get_state(&worker, &parties).await;
    let fpo_storage =
        state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    let nft_collection_storage = state_after.nft.storage_usage - state_before.nft.storage_usage;
    seller_tokens_burnt += get_tokens_burnt(&outcome);
    println!(
        "Fixed Price Offering listing added, collection ID {}",
        collection_id
    );

    /*
    #01: Proposed price too low
    */
    /*    println!(
        "{}: Proposed price too low:",
        "#01 fpo_place_proposal".cyan()
    );
    let state_before = state_after;
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
    let state_after = get_state(&worker, &parties).await;
    tokens_burnt_buyer1 += state_before.buyer1.balance - state_after.buyer1.balance;

    println!(" - {}", "PASSED".green());*/

    /*
    #02 Proposed price of 550yN acceptable, deposit 500yN is too low
    */
    /*    println!(
        "{}: Proposed price of 550yN acceptable, deposit 500yN is too low:",
        "#02 fpo_place_proposal".cyan()
    );
    let state_before = state_after;
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
    let state_after = get_state(&worker, &parties).await;
    tokens_burnt_buyer1 += state_before.buyer1.balance - state_after.buyer1.balance;

    println!(" - {}", "PASSED".green());*/

    /*
    #03 Buyer 1 proposed price of 550yN acceptable, deposit sufficient
    */
    println!(
        "{}: Buyer 1 proposed price of 550yN acceptable, deposit sufficient:",
        "#03 fpo_place_proposal".cyan()
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
    let proposal1_storage =
        state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    verify_balances(&outcome, &state_before, &state_after, proposal1_price, 0);
    buyer1_tokens_burnt += get_tokens_burnt(&outcome);

    println!(" - {}", "PASSED".green());

    /*
    #04 Buyer 2 proposed price of 600yN acceptable, deposit sufficient
    */
    println!(
        "{}: Buyer 2 proposed price of 600yN acceptable, deposit sufficient:",
        "#04 fpo_place_proposal".cyan()
    );

    let proposal2_price: Balance = 600;
    let state_before = get_state(&worker, &parties).await;

    let outcome = buyer2_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "600",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(proposal2_price + fpo_place_proposal_worst_case_storage_cost)
        .transact()
        .await?;
    let to_be_revoked_proposal_id = outcome.json::<u64>()?;
    let state_after = get_state(&worker, &parties).await;
    let proposal2_storage =
        state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    verify_balances(&outcome, &state_before, &state_after, proposal2_price, 0);
    buyer2_tokens_burnt += get_tokens_burnt(&outcome);

    println!(" - {}", "PASSED".green());

    /*
    #05 Buyer 2 revokes their proposal
    */
    println!(
        "{}: Buyer 2 revokes their proposal",
        "#05 fpo_revoke_proposal".cyan()
    );

    let state_before = get_state(&worker, &parties).await;

    let outcome = buyer2_account
        .call(&worker, marketplace_contract.id(), "fpo_revoke_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "proposal_id": to_be_revoked_proposal_id,
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(0)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;
    let revoke_fee = proposal2_price * FPO_ACCEPTING_PROPOSALS_REVOKE_FEE_RATE / 100;
    let revoke_refund = proposal2_price - revoke_fee;
    verify_balances(&outcome, &state_before, &state_after, 0, revoke_refund);
    let expected_fees_balance = state_before.fees.balance + revoke_fee;
    assert_eq!(
        state_after.fees.balance, expected_fees_balance,
        "Deducted fee is incorrect"
    );
    buyer2_tokens_burnt += get_tokens_burnt(&outcome);

    println!(" - {}", "PASSED".green());

    /*
    #06 Buyer 2 re-submits their proposal at 600yN
    */
    println!(
        "{}: Buyer 2 re-submits their proposal at 600yN:",
        "#06 fpo_place_proposal".cyan()
    );

    let proposal2_price: Balance = 600;
    let state_before = get_state(&worker, &parties).await;

    let outcome = buyer2_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "600",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(proposal2_price + fpo_place_proposal_worst_case_storage_cost)
        .transact()
        .await?;
    let state_after = get_state(&worker, &parties).await;
    let proposal2_storage =
        state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    verify_balances(&outcome, &state_before, &state_after, proposal2_price, 0);
    buyer2_tokens_burnt += get_tokens_burnt(&outcome);

    println!(" - {}", "PASSED".green());

    /*
    #07 Buyer 3 proposed price of 510yN expected to be rejected
    */
    println!(
        "{}: Buyer 3 proposed price of 510yN expected to be rejected",
        "#07 fpo_place_proposal case".cyan()
    );

    let state_before = get_state(&worker, &parties).await;

    let outcome = buyer3_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "510",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(510 + fpo_place_proposal_worst_case_storage_cost)
        .transact()
        .await;
    assert!(
        outcome.is_err(),
        "Accepted even though should have been rejected"
    );
    let state_after = get_state(&worker, &parties).await;
    buyer3_tokens_burnt += state_before.buyer3.balance - state_after.buyer3.balance;

    println!(" - {}", "PASSED".green());

    /*
    #08 Buyer 3 proposed price of 700yN, outbids buyer 1 at 550yN
    */
    println!(
        "{}: Buyer 3 proposed price of 700yN, outbids buyer 1 at 550yN",
        "#08 fpo_place_proposal".cyan()
    );

    let proposal3_price: Balance = 700;
    let state_before = get_state(&worker, &parties).await;

    let outcome = buyer3_account
        .call(&worker, marketplace_contract.id(), "fpo_place_proposal")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
            "price_yocto": "700",
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_PLACE_GAS)
        .deposit(proposal3_price + fpo_place_proposal_worst_case_storage_cost)
        .transact()
        .await?;
    let state_after = get_state(&worker, &parties).await;
    let proposal3_storage =
        state_after.marketplace.storage_usage - state_before.marketplace.storage_usage;
    verify_balances(
        &outcome,
        &state_before,
        &state_after,
        proposal3_price,
        proposal1_price,
    );
    buyer3_tokens_burnt += get_tokens_burnt(&outcome);
    let buyer1_refund = state_after.buyer1.balance - state_before.buyer1.balance;
    let buyer1_storage_refund = buyer1_refund - proposal1_price;
    let buyer1_storage_freed =
        (buyer1_storage_refund / STORAGE_COST_YOCTO_PER_BYTE as Balance) as u64;
    let proposal3_storage = state_after.marketplace.storage_usage
        - state_before.marketplace.storage_usage
        + buyer1_storage_freed;

    assert_eq!(
        state_after.buyer1.balance,
        state_before.buyer1.balance
            + proposal1_price
            + proposal1_storage as Balance * STORAGE_COST_YOCTO_PER_BYTE,
        "Buyer1 refund incorrect"
    );

    println!(" - {}", "PASSED".green());

    /*
    #09 Buyer 3 buys one item at buy_now price outbidding buyer 2 bid at 600yN
    */
    println!(
        "{}: Buyer 3 buys one item at buy_now price outbidding buyer 2 bid at 600yN",
        "#09 fpo_buy".cyan()
    );

    let state_before = get_state(&worker, &parties).await;

    let nft_mint_worst_case_storage_cost =
        NFT_MINT_WORST_CASE_STORAGE as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let outcome = buyer3_account
        .call(&worker, marketplace_contract.id(), "fpo_buy")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_BUY_GAS)
        .deposit(1000 + nft_mint_worst_case_storage_cost)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;
    assert!(
        state_after.seller.balance == state_before.seller.balance + 1000,
        "Seller wasn't paid properly"
    );

    verify_balances(&outcome, &state_before, &state_after, 1000, proposal2_price);
    assert_eq!(
        state_after.buyer2.balance,
        state_before.buyer2.balance
            + proposal2_price
            + proposal2_storage as Balance * STORAGE_COST_YOCTO_PER_BYTE,
        "Buyer2 refund incorrect"
    );
    buyer3_tokens_burnt += get_tokens_burnt(&outcome);
    let nft_mint_storage = state_after.nft.storage_usage - state_before.nft.storage_usage;

    println!(" - {}", "PASSED".green());

    /*
    #10 Seller concludes the offering, all deposits get returned
    */
    println!(
        "{}: Seller concludes the offering, all pending deposits get returned",
        "#10 fpo_conclude".cyan()
    );

    let state_before = get_state(&worker, &parties).await;

    let outcome = seller_account
        .call(&worker, marketplace_contract.id(), "fpo_conclude")
        .args_json(json!({
            "nft_contract_id": nft_account.id().clone(),
            "collection_id": collection_id,
        }))?
        .gas(FPO_ACCEPTING_PROPOSALS_CONCLUDE_GAS)
        .deposit(0)
        .transact()
        .await?;

    let state_after = get_state(&worker, &parties).await;

    // check seller balance
    let tokens_burnt = get_tokens_burnt(&outcome);
    seller_tokens_burnt += get_tokens_burnt(&outcome);
    let expected_seller_refund = fpo_storage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    assert_eq!(
        state_after.seller.balance,
        state_before.seller.balance + expected_seller_refund - tokens_burnt,
        "Seller refund incorrect"
    );

    // check buyer3 balance
    let expected_buyer3_refund =
        proposal3_price + proposal3_storage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    assert_eq!(
        state_after.buyer3.balance,
        state_before.buyer3.balance + expected_buyer3_refund,
        "Buyer3 refundc incorrect"
    );

    // check balance changes over the entire sequence of operations
    let state_final = state_after;

    let nft_collection_storage_cost =
        nft_collection_storage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let expected_seller_balance = state_initial.seller.balance - seller_tokens_burnt - nft_collection_storage_cost + 1000/*buy_now*/;
    assert_eq!(
        state_final.seller.balance, expected_seller_balance,
        "Seller balance overall reconciliation failed"
    );

    let expected_fees_balance = state_initial.fees.balance + revoke_fee;
    assert_eq!(
        state_final.fees.balance, expected_fees_balance,
        "Fees balance overall reconciliation failed"
    );

    let expected_buyer1_balance = state_initial.buyer1.balance - buyer1_tokens_burnt;
    assert_eq!(
        state_final.buyer1.balance, expected_buyer1_balance,
        "Buyer1 balance overall reconciliation failed"
    );

    let expected_buyer2_balance = state_initial.buyer2.balance - buyer2_tokens_burnt - revoke_fee;
    assert_eq!(
        state_final.buyer2.balance, expected_buyer2_balance,
        "Buyer2 balance overall reconciliation failed"
    );

    let nft_mint_storage_cost = nft_mint_storage as Balance * STORAGE_COST_YOCTO_PER_BYTE;
    let expected_buyer3_balance = state_initial.buyer3.balance - buyer3_tokens_burnt - 1000/*buy_now*/ - nft_mint_storage_cost;
    assert_eq!(
        state_final.buyer3.balance, expected_buyer3_balance,
        "Buyer3 balance overall reconciliation failed",
    );

    assert_eq!(
        state_initial.marketplace.storage_usage, state_final.marketplace.storage_usage,
        "Not all marketplace storage has been freed"
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
    let fees_info = parties
        .fees
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
        fees: fees_info,
        nft: nft_info,
        seller: seller_info,
        buyer1: buyer1_info,
        buyer2: buyer2_info,
        buyer3: buyer3_info,
    }
}

fn get_tokens_burnt(execution_details: &CallExecutionDetails) -> Balance {
    execution_details
        .receipt_outcomes()
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + execution_details.outcome().tokens_burnt
}

// checks if buyers' balances are correctly explained by gas spendings, marketplace and nft storage changes
// and extra proposal deposits placed and returned
fn verify_balances(
    execution_details: &CallExecutionDetails,
    state_before: &State,
    state_after: &State,
    marketplace_deposit_placed: Balance,
    marketplace_deposit_returned: Balance,
) {
    let transaction = execution_details.outcome();
    let receipts = execution_details.receipt_outcomes();

    // storage
    // let nft_storage_usage = state_after.nft.storage_usage - state_before.nft.storage_usage;
    // let nft_storage_cost = nft_storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE;

    // gas
    let buyer_gas_cost = receipts
        .iter()
        .fold(0, |acc, receipt| acc + receipt.tokens_burnt)
        + transaction.tokens_burnt;

    // here we make sure that all the cost resulting from marketplace storage difference gets
    // covered by buyers or refunded the buyers
    assert_eq!(
        state_before.buyer1.balance
            + state_before.buyer2.balance
            + state_before.buyer3.balance
            + marketplace_deposit_returned
            + state_before.marketplace.storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE
            + state_before.nft.storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE
            - state_after.marketplace.storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE
            - state_after.nft.storage_usage as Balance * STORAGE_COST_YOCTO_PER_BYTE
            - buyer_gas_cost
            - marketplace_deposit_placed,
        state_after.buyer1.balance + state_after.buyer2.balance + state_after.buyer3.balance
    );
}
