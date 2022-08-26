#[cfg(test)]
mod internal_tests {
    use crate::*;
    use crate::FixedPriceOffering;
    use crate::FixedPriceOfferingProposal;
    use crate::FixedPriceOfferingStatus::*;
    use crate::fpo::config::*;
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::collections::{LookupMap, Vector};
    use near_sdk::{testing_env, AccountId, VMContext};

    #[test]
    fn test_lifetime() {
        let offeror_id_string = "offeror.eneftigo.testnet".to_string();

        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            8460000000000000000000,
            0,
        );
        testing_env!(context);
        
        let mut fpo = test_fpo(
            "nft.eneftigo.testnet", 
            0,
            "offeror.eneftigo.testnet", 
            Some("1975-05-26T00:00:00+00:00"), 
            Some("1975-06-24T00:00:00+00:00")
        );

        // move date forward but not reach start date yet
        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 5, 25).and_hms(00, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        fpo.update_status();
        assert!(fpo.status == Unstarted);

        // move date beyond start date
        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 5, 26).and_hms(01, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        fpo.update_status();
        assert!(fpo.status == Running);

        // move date back before start date, it should remain in a Running state
        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 5, 25).and_hms(01, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        fpo.update_status();
        assert!(fpo.status == Running);

        // move date back beyond end date
        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 6, 25).and_hms(01, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        fpo.update_status();
        assert!(fpo.status == Ended);

        // move date back before end date, it should remain Ended
        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 6, 20).and_hms(01, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        fpo.update_status();
        assert!(fpo.status == Ended);

        // move date back before start date, it should remain Ended
        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 5, 20).and_hms(01, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);
        fpo.update_status();
        assert!(fpo.status == Ended);        
    }

    #[test]
    fn test_functions() {
        let offeror_id_string = "offeror.eneftigo.testnet".to_string();

        let context = get_context(
            offeror_id_string.clone(),
            offeror_id_string.clone(),
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            8460000000000000000000,
            0,
        );
        testing_env!(context);
        
        let mut fpo = test_fpo(
            "nft.eneftigo.testnet", 
            0,
            "offeror.eneftigo.testnet", 
            Some("1975-05-26T00:00:00+00:00"), 
            Some("1975-06-24T00:00:00+00:00")
        );

        // prepare proposals
        let proposal1 = FixedPriceOfferingProposal {
            id: 1,
            proposer_id: AccountId::new_unchecked("proposer1".to_string()),
            price_yocto: 900,
            is_acceptable: true,
        };
        fpo.proposals.insert(&proposal1.id, &proposal1);
        fpo.acceptable_proposals.push(&proposal1.id);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal2 = FixedPriceOfferingProposal {
            id: 2,
            proposer_id: AccountId::new_unchecked("proposer2".to_string()),
            price_yocto: 600,
            is_acceptable: true,
        };
        fpo.proposals.insert(&proposal2.id, &proposal2);
        fpo.acceptable_proposals.push(&proposal2.id);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal3 = FixedPriceOfferingProposal {
            id: 3,
            proposer_id: AccountId::new_unchecked("proposer3".to_string()),
            price_yocto: 800,
            is_acceptable: true,
        };
        fpo.proposals.insert(&proposal3.id, &proposal3);
        fpo.acceptable_proposals.push(&proposal3.id);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal5 = FixedPriceOfferingProposal {
            id: 5,
            proposer_id: AccountId::new_unchecked("proposer5".to_string()),
            price_yocto: 500,
            is_acceptable: true,
        };
        fpo.proposals.insert(&proposal5.id, &proposal5);
        fpo.acceptable_proposals.push(&proposal5.id);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal4 = FixedPriceOfferingProposal {
            id: 4,
            proposer_id: AccountId::new_unchecked("proposer4".to_string()),
            price_yocto: 500,
            is_acceptable: true,
        };
        fpo.proposals.insert(&proposal4.id, &proposal4);
        fpo.acceptable_proposals.push(&proposal4.id);

        assert!(
            fpo.acceptable_proposals.get(0) == Some(proposal1.id) &&
            fpo.acceptable_proposals.get(1) == Some(proposal2.id) &&
            fpo.acceptable_proposals.get(2) == Some(proposal3.id) &&
            fpo.acceptable_proposals.get(3) == Some(proposal5.id) &&
            fpo.acceptable_proposals.get(4) == Some(proposal4.id),
            "Wrong initial ordering"
        );

        // test sort_acceptable_proposals
        fpo.sort_acceptable_proposals();
        assert!(
            fpo.acceptable_proposals.get(0) == Some(proposal5.id) &&
            fpo.acceptable_proposals.get(1) == Some(proposal4.id) &&
            fpo.acceptable_proposals.get(2) == Some(proposal2.id) &&
            fpo.acceptable_proposals.get(3) == Some(proposal3.id) &&
            fpo.acceptable_proposals.get(4) == Some(proposal1.id),
            "Wrong sorted ordering"
        );

        assert!(
            fpo.acceptable_price_yocto() == (500 + PRICE_STEP_YOCTO),
            "Wrong acceptable price"
        );

        // test prune_supply_exceeding_acceptable_proposals
        fpo.supply_left = 3;
        fpo.prune_supply_exceeding_acceptable_proposals();
        assert!(
            fpo.acceptable_proposals.len() == 3,
            "Number of acceptable proposals after pruning does not match supply"
        );
        assert!(
            fpo.acceptable_proposals.get(0) == Some(proposal2.id) &&
            fpo.acceptable_proposals.get(1) == Some(proposal3.id) &&
            fpo.acceptable_proposals.get(2) == Some(proposal1.id),
            "Wrong proposals pruned"
        );
        assert!(
            fpo.acceptable_price_yocto() == (600 + PRICE_STEP_YOCTO),
            "Wrong acceptable price"
        );

        // test is_proposal_acceptable
        assert!(
            fpo.is_proposal_acceptable(proposal2.id) &&
            fpo.is_proposal_acceptable(proposal3.id) &&
            fpo.is_proposal_acceptable(proposal1.id) &&
            !fpo.is_proposal_acceptable(proposal4.id) &&
            !fpo.is_proposal_acceptable(proposal5.id),
            "Wrong proposals are acceptable/unacceptable"
        );

        // test acceptable_price_yocto
    }

    /*
     * Helpers
     */

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

    // fn test_nft_metadata(index: i32) -> TokenMetadata {
    //     TokenMetadata {
    //         title: Some(format!("nft{}", index)),
    //         description: None,
    //         media: None,
    //         media_hash: None,
    //         copies: Some(1),
    //         issued_at: None,
    //         expires_at: None,
    //         starts_at: None,
    //         updated_at: None,
    //         extra: None,
    //         reference: None,
    //         reference_hash: None,
    //     }
    // }

    fn test_fpo(
        nft_contract_id_str: &str,
        collection_id: NftCollectionId,
        offeror_id_str: &str,
        start_date: Option<&str>, 
        end_date: Option<&str>
    ) -> FixedPriceOffering {
        let start_timestamp: Option<i64> = if let Some(start_date) = start_date {
            Some(DateTime::parse_from_rfc3339(start_date).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            ).timestamp_nanos())
        } else {
            None
        };
        let end_timestamp = if let Some(end_date) = end_date {
            Some(DateTime::parse_from_rfc3339(end_date).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            ).timestamp_nanos())
        } else {
            None
        };

        let nft_contract_id = AccountId::new_unchecked(nft_contract_id_str.to_string());
        let offering_id = OfferingId { nft_contract_id, collection_id };
        let offeror_id = AccountId::new_unchecked(offeror_id_str.to_string());

        FixedPriceOffering {
            offering_id: offering_id,
            offeror_id: offeror_id,
            supply_total: 5,
            buy_now_price_yocto: 1000,
            min_proposal_price_yocto: Some(500),
            start_timestamp: start_timestamp,
            end_timestamp: end_timestamp,
            status: Unstarted,
            // nft_metadata: test_nft_metadata(1),
            supply_left: 5,
            proposals: LookupMap::new(b"m"),
            proposals_by_proposer: LookupMap::new(b"p"),
            acceptable_proposals: Vector::new(b"a"),
            next_proposal_id: 0,
        }
    }
}