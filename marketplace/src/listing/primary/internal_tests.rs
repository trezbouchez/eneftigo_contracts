#[cfg(test)]
mod internal_tests {
    use crate::{
        *,
        external::{NftMetadata},
        listing::{
            constants::*,
            status::{ListingStatus},
            proposal::{Proposal},
            primary::lib::{PrimaryListing},
        },
    };
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::collections::{Vector};
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
            Some("1975-06-24T00:00:00+00:00"),
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
        assert!(fpo.status == ListingStatus::Unstarted);

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
        assert!(fpo.status == ListingStatus::Running);

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
        assert!(fpo.status == ListingStatus::Running);

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
        assert!(fpo.status == ListingStatus::Ended);

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
        assert!(fpo.status == ListingStatus::Ended);

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
        assert!(fpo.status == ListingStatus::Ended);
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
            Some("1975-06-24T00:00:00+00:00"),
        );

        // prepare proposals
        let proposal1 = Proposal {
            id: 1,
            proposer_id: AccountId::new_unchecked("proposer1".to_string()),
            price_yocto: 900,
        };
        fpo.proposals.push(&proposal1);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal2 = Proposal {
            id: 2,
            proposer_id: AccountId::new_unchecked("proposer2".to_string()),
            price_yocto: 600,
        };
        fpo.proposals.push(&proposal2);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal3 = Proposal {
            id: 3,
            proposer_id: AccountId::new_unchecked("proposer3".to_string()),
            price_yocto: 800,
        };
        fpo.proposals.push(&proposal3);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal4 = Proposal {
            id: 4,
            proposer_id: AccountId::new_unchecked("proposer4".to_string()),
            price_yocto: 500,
        };
        fpo.proposals.push(&proposal4);
        assert!(
            fpo.acceptable_price_yocto() == 500,
            "Wrong acceptable price"
        );

        let proposal5 = Proposal {
            id: 5,
            proposer_id: AccountId::new_unchecked("proposer5".to_string()),
            price_yocto: 500,
        };
        fpo.proposals.push(&proposal5);

        assert!(
            fpo.proposals.get(0).unwrap().id == proposal1.id
                && fpo.proposals.get(1).unwrap().id == proposal2.id
                && fpo.proposals.get(2).unwrap().id == proposal3.id
                && fpo.proposals.get(3).unwrap().id == proposal4.id
                && fpo.proposals.get(4).unwrap().id == proposal5.id,
            "Wrong initial ordering"
        );

        // test sort_acceptable_proposals
        fpo.sort_proposals();
        assert!(
            fpo.proposals.get(0).unwrap().id == proposal1.id
                && fpo.proposals.get(1).unwrap().id == proposal3.id
                && fpo.proposals.get(2).unwrap().id == proposal2.id
                && fpo.proposals.get(3).unwrap().id == proposal4.id
                && fpo.proposals.get(4).unwrap().id == proposal5.id,
            "Wrong sorted ordering",
        );

        assert!(
            fpo.acceptable_price_yocto() == (500 + PRICE_STEP_YOCTO),
            "Wrong acceptable price"
        );

        // test prune_supply_exceeding_acceptable_proposals
        fpo.supply_left = 3;

        // fpo.remove_supply_exceeding_proposals_and_refund_proposers();
        assert!(
            fpo.proposals.len() == 3,
            "Number of acceptable proposals after pruning does not match supply"
        );
        assert!(
            fpo.proposals.get(0).unwrap().id == proposal1.id
                && fpo.proposals.get(1).unwrap().id == proposal3.id
                && fpo.proposals.get(2).unwrap().id == proposal2.id,
            "Wrong proposals pruned"
        );
        assert!(
            fpo.acceptable_price_yocto() == (600 + PRICE_STEP_YOCTO),
            "Wrong acceptable price"
        );

        // test is_proposal_acceptable
        assert!(fpo.proposals.len() == 3, "Wrong proposals are acceptable/unacceptable");

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
        end_date: Option<&str>,
    ) -> PrimaryListing {
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
        let offering_id = PrimaryListingId {
            nft_contract_id,
            collection_id: collection_id,
        };
        let offeror_id = AccountId::new_unchecked(offeror_id_str.to_string());
        let nft_metadata = NftMetadata::new(
            &String::from("Bored Grapes"),
            &String::from("https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF"),
        );
        PrimaryListing {
            id: offering_id,
            seller_id: offeror_id,
            nft_metadata,
            supply_total: 5,
            buy_now_price_yocto: 1000,
            min_proposal_price_yocto: Some(500),
            start_timestamp: start_timestamp,
            end_timestamp: end_timestamp,
            status: ListingStatus::Unstarted,
            // nft_metadata: test_nft_metadata(1),
            supply_left: 5,
            proposals: Vector::new(b"m"),
            next_proposal_id: 0,
        }
    }
}
