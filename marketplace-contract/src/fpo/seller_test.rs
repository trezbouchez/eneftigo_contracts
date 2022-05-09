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

    #[test]
    #[should_panic(expected = r#"Prices must be integer multiple of 10 yoctoNear"#)]
    fn test_add_buy_now_price_not_multiple_of_step() {
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
            U128(1115),                              // buy_now_price_yocto
            U128(50),                                // min_proposal_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Prices must be integer multiple of 10 yoctoNear"#)]
    fn test_add_min_proposal_price_not_multiple_of_step() {
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
            U128(1200),                              // buy_now_price_yocto
            U128(55),                                // min_proposal_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Min proposal price must be lower than buy now price"#)]
    fn test_add_buy_now_price_not_higher_than_min_proposal_price() {
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
            U128(1100),                              // buy_now_price_yocto
            U128(1100),                              // min_proposal_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"End date is into the past"#)]
    fn test_end_date_into_the_past() {
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
            U128(1100),                              // buy_now_price_yocto
            U128(500),                               // min_proposal_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-04-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Start date is into the past"#)]
    fn test_start_date_into_the_past() {
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
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            Some("1975-04-24T00:00:00+00:00".to_string()),  // start_date
            "1975-06-24T00:00:00+00:00".to_string(),        // end_date
        );
    }    

    #[test]
    #[should_panic(expected = r#"Offering duration too short"#)]
    fn test_duration_too_short() {
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
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()),  // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }   

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_supply_of_zero() {
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
            0,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_supply_too_many() {
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
            101,                                            // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  
 
     
    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_wrong_end_date_format() {
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
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-5-24T13:50:00+00:00".to_string(),         // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_wrong_start_date_format() {
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
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            Some("1975-05-24".to_string()),                 // start_date
            "1975-5-24T13:50:00+00:00".to_string(),         // end_date
        );
    }
    
    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_already_listed() {
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
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            U128(50),                                       // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-06-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  


    /*
     * fpo_add_buy_now_only
     */

    #[test]
    #[should_panic(expected = r#"Price cannot be lower than 1000 yoctoNear"#)]
    fn test_buy_now_add_buy_now_price_too_low() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                                       // total_supply
            U128(800),                               // buy_now_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,
            None
        );
    }

    #[test]
    #[should_panic(expected = r#"Price must be integer multiple of 10 yoctoNear"#)]
    fn test_buy_now_add_buy_now_price_not_multiple_of_step() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                                       // total_supply
            U128(1115),                              // buy_now_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,
            None
        );
    }

    #[test]
    #[should_panic(expected = r#"End date is into the past"#)]
    fn test_buy_now_end_date_into_the_past() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                                       // total_supply
            U128(1100),                              // buy_now_price_yocto
            test_nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            Some("1975-04-24T00:00:00+00:00".to_string()), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Start date is into the past"#)]
    fn test_buy_now_start_date_into_the_past() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            Some("1975-04-24T00:00:00+00:00".to_string()),  // start_date
            None
        );
    }    

    #[test]
    #[should_panic(expected = r#"Offering duration too short"#)]
    fn test_buy_now_duration_too_short() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()),  // start_date
            Some("1975-05-24T13:50:00+00:00".to_string()),  // end_date
        );
    }   

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_buy_now_supply_of_zero() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            0,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_buy_now_supply_too_many() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            101,                                            // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  
 
     
    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_buy_now_wrong_end_date_format() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            Some("1975-5-24T13:50:00+00:00".to_string()),   // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_buy_now_wrong_start_date_format() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            Some("1975-05-24".to_string()),                 // start_date
            None
        );
    }
    
    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_buy_now_already_listed() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,
            None
        );

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  

    /*
     * Other
     */

    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_already_listed_proposal_vs_buy_now() {
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
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  

    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_already_listed_buy_now_vs_proposal() {
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

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,
            None
        );

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(nft_contract_id.clone()),
            AccountId::new_unchecked(offeror_id.clone()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            test_nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  
}
