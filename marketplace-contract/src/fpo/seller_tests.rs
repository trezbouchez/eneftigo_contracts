#[cfg(test)]
mod seller_tests {
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::json_types::U128;
    use near_sdk::{testing_env, AccountId, VMContext};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::collections::{LookupMap, UnorderedSet, Vector};
    use crate::{MarketplaceStorageKey, MarketplaceContract, TokenMetadata};
    use crate::ProposalId;
    use crate::FixedPriceOffering;
    use crate::FixedPriceOfferingProposal;
    use crate::FixedPriceOfferingStorageKey;
    use crate::FixedPriceOfferingStatus::*;
    use crate::internal::hash_account_id;
    use near_sdk::borsh::BorshSerialize;

    const MARKETPLACE_ACCOUNT_ID: &str = "marketplace.eneftigo.testnet";
    const NFT_ACCOUNT_ID: &str = "nft.eneftigo.testnet";
    const OFFEROR_ACCOUNT_ID: &str = "offeror.eneftigo.testnet";
    const MALICIOUS_ACCOUNT_ID: &str = "malicious.eneftigo.testnet";
    const PROPOSER1_ACCOUNT_ID: &str = "proposer1.eneftigo.testnet";
    const PROPOSER2_ACCOUNT_ID: &str = "proposer2.eneftigo.testnet";

    /*
     * fpo_add_accepting_proposals assertions
     */

    #[test]
    #[should_panic(expected = r#"Price cannot be lower than 1000 yoctoNear"#)]
    fn test_add_buy_now_price_too_low() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        
        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(800),                               // buy_now_price_yocto
            U128(50),                                // min_proposal_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Prices must be integer multiple of 10 yoctoNear"#)]
    fn test_add_buy_now_price_not_multiple_of_step() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(1115),                              // buy_now_price_yocto
            U128(50),                                // min_proposal_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Prices must be integer multiple of 10 yoctoNear"#)]
    fn test_add_min_proposal_price_not_multiple_of_step() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(1200),                              // buy_now_price_yocto
            U128(55),                                // min_proposal_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Min proposal price must be lower than buy now price"#)]
    fn test_add_buy_now_price_not_higher_than_min_proposal_price() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        
        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(1100),                              // buy_now_price_yocto
            U128(1100),                              // min_proposal_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"End date is into the past"#)]
    fn test_end_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(1100),                              // buy_now_price_yocto
            U128(500),                               // min_proposal_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            "1975-04-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Start date is into the past"#)]
    fn test_start_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-04-24T00:00:00+00:00".to_string()),  // start_date
            "1975-06-24T00:00:00+00:00".to_string(),        // end_date
        );
    }    

    #[test]
    #[should_panic(expected = r#"Offering duration too short"#)]
    fn test_duration_too_short() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()),  // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }   

    #[test]
    #[should_panic(expected = r#"Offering duration too long"#)]
    fn test_duration_too_long() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()),  // start_date
            "1975-06-15T13:50:00+00:00".to_string(),        // end_date
        );
    }   

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_supply_of_zero() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            0,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_supply_too_many() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            101,                                            // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  
 
     
    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_wrong_end_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-5-24T13:50:00+00:00".to_string(),         // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_wrong_start_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-05-24".to_string()),                 // start_date
            "1975-5-24T13:50:00+00:00".to_string(),         // end_date
        );
    }
    
    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_already_listed() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            U128(50),                                       // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-06-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  


    /*
     * fpo_add_buy_now_only assertions
     */

    #[test]
    #[should_panic(expected = r#"Price cannot be lower than 1000 yoctoNear"#)]
    fn test_buy_now_add_buy_now_price_too_low() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(800),                               // buy_now_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,
            None
        );
    }

    #[test]
    #[should_panic(expected = r#"Price must be integer multiple of 10 yoctoNear"#)]
    fn test_buy_now_add_buy_now_price_not_multiple_of_step() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(1115),                              // buy_now_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,
            None
        );
    }

    #[test]
    #[should_panic(expected = r#"End date is into the past"#)]
    fn test_buy_now_end_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                       // total_supply
            U128(1100),                              // buy_now_price_yocto
            nft_metadata(1),                    // nft_metadata
            None,                                    // start_date
            Some("1975-04-24T00:00:00+00:00".to_string()), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Start date is into the past"#)]
    fn test_buy_now_start_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-04-24T00:00:00+00:00".to_string()),  // start_date
            None
        );
    }    

    #[test]
    #[should_panic(expected = r#"Offering duration too short"#)]
    fn test_buy_now_duration_too_short() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            2,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()),  // start_date
            Some("1975-05-24T13:50:00+00:00".to_string()),  // end_date
        );
    }   

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_buy_now_supply_of_zero() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            0,                                              // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_buy_now_supply_too_many() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            101,                                            // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  
 
     
    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_buy_now_wrong_end_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            Some("1975-5-24T13:50:00+00:00".to_string()),   // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#)]
    fn test_buy_now_wrong_start_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            Some("1975-05-24".to_string()),                 // start_date
            None
        );
    }
    
    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_buy_now_already_listed() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,
            None
        );

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  

    /*
     * proposal_accepting vs buy_now_only assertions
     */

    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_already_listed_proposal_vs_buy_now() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,
            None
        );
    }  

    #[test]
    #[should_panic(expected = r#"Already listed"#)]
    fn test_already_listed_buy_now_vs_proposal() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            10,                                             // total_supply
            U128(2000),                                     // buy_now_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,
            None
        );

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string()),
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()),
            50,                                             // total_supply
            U128(1100),                                     // buy_now_price_yocto
            U128(500),                                      // min_proposal_price_yocto
            nft_metadata(1),                           // nft_metadata
            None,                                           // start_date
            "1975-05-24T13:50:00+00:00".to_string(),        // end_date
        );
    }  

    /* 
     * Accepting proposals
     */

    #[test]
    #[should_panic(expected = r#"Only the offeror can accept proposals"#)]
    fn test_accepting_proposals_by_unauthorized_user() {
        let context = test_get_context(
            true,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);

        marketplace.fpos_by_contract_id.insert(&nft_account_id, &fpo);
        let fpos_by_this_offeror = UnorderedSet::new(
            MarketplaceStorageKey::FposByOfferorIdInner { account_id_hash: hash_account_id(&offeror_account_id) }.try_to_vec().unwrap(),
        );
        marketplace.fpos_by_offeror_id.insert(&offeror_account_id, &fpos_by_this_offeror);

        marketplace.fpo_accept_proposals(nft_account_id.clone(), 1);
    }

    #[test]
    #[should_panic(expected = r#"There's not enough proposals (3)"#)]
    fn test_accepting_too_many_proposals() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);

        marketplace.fpos_by_contract_id.insert(&nft_account_id, &fpo);
        let fpos_by_this_offeror = UnorderedSet::new(
            MarketplaceStorageKey::FposByOfferorIdInner { account_id_hash: hash_account_id(&offeror_account_id) }.try_to_vec().unwrap(),
        );
        marketplace.fpos_by_offeror_id.insert(&offeror_account_id, &fpos_by_this_offeror);

        marketplace.fpo_accept_proposals(nft_account_id.clone(), 4);
    }

    #[test]
    fn test_accepting_proposals() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);

        marketplace.fpos_by_contract_id.insert(&nft_account_id, &fpo);
        let fpos_by_this_offeror = UnorderedSet::new(
            MarketplaceStorageKey::FposByOfferorIdInner { account_id_hash: hash_account_id(&offeror_account_id) }.try_to_vec().unwrap(),
        );
        marketplace.fpos_by_offeror_id.insert(&offeror_account_id, &fpos_by_this_offeror);

        marketplace.fpo_accept_proposals(nft_account_id.clone(), 1);
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id).expect("Could not get updated FPO");
        assert!(
            fpo.acceptable_proposals.len() == 2,
            "Wrong number of acceptable_proposals"
        );
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![1,3],
            "acceptable_proposals content incorrect"
        );
        assert!(
            fpo.proposals.get(&1).is_some() && fpo.proposals.get(&2).is_none() && fpo.proposals.get(&3).is_some(),
            "proposals content incorrect"
        );
        let proposals_by_proposer1 = fpo.proposals_by_proposer.get(&AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string())).unwrap();
        assert!(
            proposals_by_proposer1.contains(&1),
            "proposals_by_proposer incorrect for proposer1"
        );
        let proposals_by_proposer2 = fpo.proposals_by_proposer.get(&AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string())).unwrap();
        assert!(
            !proposals_by_proposer2.contains(&2) && proposals_by_proposer2.contains(&3),
            "proposals_by_proposer incorrect for proposer2"
        );
        assert!(
            fpo.supply_left == 2,
            "Wrong supply_left"
        );

        marketplace.fpo_accept_proposals(nft_account_id.clone(), 2);
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id).expect("Could not get updated FPO");
        assert!(
            fpo.acceptable_proposals.is_empty(),
            "Wrong number of acceptable_proposals"
        );
        assert!(
            fpo.proposals.get(&1).is_none() && fpo.proposals.get(&2).is_none() && fpo.proposals.get(&3).is_none(),
            "proposals content incorrect"
        );
        let proposals_by_proposer1 = fpo.proposals_by_proposer.get(&AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string()));
        assert!(
            proposals_by_proposer1.is_none(),
            "proposals_by_proposer exist for proposer1"
        );
        let proposals_by_proposer2 = fpo.proposals_by_proposer.get(&AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string()));
        assert!(
            proposals_by_proposer2.is_none(),
            "proposals_by_proposer exist for proposer2"
        );
        assert!(
            fpo.supply_left == 0,
            "Wrong supply_left"
        );
     }
        
    
    /*
     * Concluding FPO
     */


    /*
     * Helpers
     */

    fn test_get_context(
        malicious: bool,
        datetime: DateTime<Utc>,
        attached_deposit: u128,
        storage_usage: u64,
    ) -> VMContext {
        let account_id = if malicious {
            AccountId::new_unchecked(MALICIOUS_ACCOUNT_ID.to_string())
        } else {
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string())
        };
        VMContextBuilder::new()
        .predecessor_account_id(account_id.clone())
        .signer_account_id(account_id.clone())
        .block_timestamp(datetime.timestamp_nanos() as u64)
        .attached_deposit(attached_deposit)
        .storage_usage(storage_usage)
        .build()
    }

    fn test_marketplace() -> MarketplaceContract {
        MarketplaceContract::new(AccountId::new_unchecked(MARKETPLACE_ACCOUNT_ID.to_string()))
    }

    fn test_fpo() -> FixedPriceOffering {
        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        let nft_account_id_hash = hash_account_id(&nft_account_id);
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());
        let start_timestamp = DateTime::parse_from_rfc3339("1975-05-26T00:00:00+00:00").unwrap().timestamp_nanos();
        let end_timestamp = DateTime::parse_from_rfc3339("1975-06-10T00:00:00+00:00").unwrap().timestamp_nanos();
        let fpo = FixedPriceOffering {
            nft_contract_id: nft_account_id.clone(),
            offeror_id: offeror_account_id.clone(),
            supply_total: 3,
            buy_now_price_yocto: 1000,
            min_proposal_price_yocto: Some(500),
            start_timestamp: Some(start_timestamp),
            end_timestamp: Some(end_timestamp),
            status: Running,
            nft_metadata: nft_metadata(1),
            supply_left: 3,
            proposals: LookupMap::new(
                FixedPriceOfferingStorageKey::Proposals { nft_contract_id_hash: nft_account_id_hash }.try_to_vec().unwrap(),
            ),
            proposals_by_proposer: LookupMap::new(
                FixedPriceOfferingStorageKey::ProposalsByProposer { nft_contract_id_hash: nft_account_id_hash }.try_to_vec().unwrap(),
            ),
            acceptable_proposals: Vector::new(
                FixedPriceOfferingStorageKey::AcceptableProposals { nft_contract_id_hash: nft_account_id_hash }.try_to_vec().unwrap(),
            ),
            next_proposal_id: 0,
        };
        
        fpo
    }

    fn test_place_proposals(fpo: &mut FixedPriceOffering) {
        let proposer1_id = AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string());
        let proposer2_id = AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string());
        let proposals_vec: Vec<FixedPriceOfferingProposal> = vec![
            FixedPriceOfferingProposal {
                id: 1,
                proposer_id: proposer1_id.clone(),
                price_yocto: 500,
                is_acceptable: true
            },
            FixedPriceOfferingProposal {
                id: 2,
                proposer_id: proposer2_id.clone(),
                price_yocto: 900,
                is_acceptable: true
            },
            FixedPriceOfferingProposal {
                id: 3,
                proposer_id: proposer2_id.clone(),
                price_yocto: 700,
                is_acceptable: true
            }
        ];
        for proposal in proposals_vec.iter() {
            fpo.proposals.insert(&proposal.id, &proposal);
        }
        fpo.acceptable_proposals.extend(vec![1,3,2]);

        let proposer1_id_hash = hash_account_id(&proposer1_id);
        let nft_account_id_hash = hash_account_id(&fpo.nft_contract_id);
        let mut proposals_by_proposer1: UnorderedSet<ProposalId> = UnorderedSet::new(
            FixedPriceOfferingStorageKey::ProposalsByProposerInner { nft_contract_id_hash: nft_account_id_hash, proposer_id_hash: proposer1_id_hash }.try_to_vec().unwrap(),
        );
        proposals_by_proposer1.extend(vec![1]);

        let proposer2_id_hash = hash_account_id(&proposer2_id);
        let mut proposals_by_proposer2: UnorderedSet<ProposalId> = UnorderedSet::new(
            FixedPriceOfferingStorageKey::ProposalsByProposerInner { nft_contract_id_hash: nft_account_id_hash, proposer_id_hash: proposer2_id_hash }.try_to_vec().unwrap(),
        );
        proposals_by_proposer2.extend(vec![2,3]);

        fpo.proposals_by_proposer.insert(&proposer1_id, &proposals_by_proposer1);
        fpo.proposals_by_proposer.insert(&proposer2_id, &proposals_by_proposer2);
    }

    fn nft_metadata(index: i32) -> TokenMetadata {
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
}