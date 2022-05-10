#[cfg(test)]
mod tests {
    use crate::{MarketplaceContract, TokenMetadata};
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::json_types::U128;
    use near_sdk::{testing_env, AccountId, VMContext};
    
    fn get_context(
        predecessor_account_id: String,
        signer_account_id: String,
        datetime: DateTime<Utc>,
        attached_deposit: u128,
        storage_usage: u64,
    ) -> VMContext {
        VMContext {
            current_account_id: "marketplace.eneftigo.testnet".to_string(),
            signer_account_id: signer_account_id,
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: datetime.timestamp_nanos() as u64,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    fn test_make_marketplace() -> MarketplaceContract {
        let marketplace_owner_id = "marketplace.eneftigo.testnet".to_string();
        let context = get_context(
            marketplace_owner_id.clone(),
            marketplace_owner_id.clone(),
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            0,
            0,
        );
        testing_env!(context);
        MarketplaceContract::new(AccountId::new_unchecked(marketplace_owner_id))
    }

    fn test_nft_metadata(index: i32) -> TokenMetadata {
        TokenMetadata {
            title: Some(format!("nft{}", index)),
            description: None,
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }


    /*
     * fpo_add_accepting_proposals
     */

    #[test]
    #[should_panic(expected = r#"Price cannot be lower than 1000 yoctoNear"#)]
    fn test_add_buy_now_price_too_low() {
        let mut marketplace = test_make_marketplace();
        let nft_contract_id = "nft.eneftigo.testnet".to_string();
        let offeror_id = "offeror.eneftigo.testnet".to_string();
        let context = get_context(
            offeror_id.clone(),
            offeror_id.clone(),
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                                       // total_supply
            U128(800),                               // buy_now_price_yocto
            U128(50),                                // min_proposal_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }


        // is_proposal_acceptable
    // Test sorting of proposals
