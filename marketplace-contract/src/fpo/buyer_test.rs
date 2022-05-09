    // Tests
    #[test]
    #[should_panic]
    fn test_is_proposal_acceptable() {
        let mut marketplace = test_make_marketplace();
        let nft_contract_id = "nft.eneftigo.testnet".to_string();
        let offeror_id = "offeror.eneftigo.testnet".to_string();
        let context = get_context(offeror_id.clone(), offeror_id.clone(), Utc.ymd(1975, 5, 24).and_hms(13, 10, 00), 8380000000000000000000, 0);
        testing_env !(context);

        marketplace.fpo_add_accepting_proposals(
            AccountId::new_unchecked(nft_contract_id),
            AccountId::new_unchecked(offeror_id),
            2,                          // total_supply
            U128(1000),                 // buy_now_price_yocto
            U128(100),                  // min_proposal_price_yocto
            test_nft_metadata(1),       // nft_metadata
            None,                       // start_date
            "1975-06-24T00:00:00+00:00".to_string() // end_date
        );
    }

        // is_proposal_acceptable
    // Test sorting of proposals
