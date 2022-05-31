use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use colored::Colorize;

const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();

    // Create Marketplace account and deploy Marketplace Contract
    let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_FILEPATH)?;
    let marketplace_contract = worker.dev_deploy(marketplace_wasm).await?;
    println!(
        "Marketplace contract deployed to {}",
        marketplace_contract.id().to_string()
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
        outcome.status.clone().as_success().is_some(),
        "Marketplace initialization failed: {:#?} {}",
        outcome.status,
        "FAILED".red()
    );
    println!(
        "Marketplace contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    // Create NFT subaccount
    let outcome = marketplace_contract
        .as_account()
        .create_subaccount(&worker, "nft")
        .initial_balance(parse_near!("5 N")) // ~3.5 is needed or deploy will fail
        .transact()
        .await?;
    assert!(
        outcome.details.status.clone().as_success().is_some(),
        "NFT subaccount creation failed: {:#?} {}",
        outcome.details.status,
        "FAILED".red()
    );
    let nft_account = outcome.result;
    println!("NFT account created at {}", nft_account.id().to_string());

    // Deploy NFT contract
    let nft_wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let outcome = nft_account.deploy(&worker, nft_wasm).await?;
    assert!(
        outcome.details.status.clone().as_success().is_some(),
        "NFT contract deployment failed: {:#?} {}",
        outcome.details.status,
        "FAILED".red()
    );
    let nft_contract = outcome.result;
    println!("NFT contract deployed to {}", nft_contract.id().to_string());

    // Initialize NFT contract
    let outcome = nft_account
        .call(&worker, nft_contract.id().clone(), "new_default_meta")
        .args_json(json!({
            "owner_id": marketplace_contract.id(),
        }))?
        .transact()
        .await?;
    assert!(
        outcome.clone().status.as_success().is_some(),
        "NFT contract initialization failed {:?} {} ",
        outcome.status,
        "FAILED".red()
    );
    println!(
        "NFT contract initialized with owner {}",
        marketplace_contract.id().to_string()
    );

    // Add FPO listing
    let seller = worker.dev_create_account().await?;
    let outcome = seller
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
    println!("Proposals-accepting Fixed Price Offering added successfully");

    // All OK
    println!("{}", "PASSED".green());

    Ok(())
}
