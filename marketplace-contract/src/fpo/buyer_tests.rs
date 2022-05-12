#[cfg(test)]
mod seller_tests {
    use crate::internal::hash_account_id;
    use crate::FixedPriceOffering;
    use crate::FixedPriceOfferingProposal;
    use crate::FixedPriceOfferingStatus::*;
    use crate::FixedPriceOfferingStorageKey;
    use crate::ProposalId;
    use crate::{MarketplaceContract, MarketplaceStorageKey, TokenMetadata};
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::collections::{LookupMap, UnorderedSet, Vector};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, VMContext};

    const MARKETPLACE_ACCOUNT_ID: &str = "marketplace.eneftigo.testnet";
    const NFT_ACCOUNT_ID: &str = "nft.eneftigo.testnet";
    const NONEXISTENT_NFT_ACCOUNT_ID: &str = "nonexistent.eneftigo.testnet";
    const OFFEROR_ACCOUNT_ID: &str = "offeror.eneftigo.testnet";
    const MALICIOUS_ACCOUNT_ID: &str = "malicious.eneftigo.testnet";
    const PROPOSER1_ACCOUNT_ID: &str = "proposer1.eneftigo.testnet";
    const PROPOSER2_ACCOUNT_ID: &str = "proposer2.eneftigo.testnet";
    const BIDDER_ACCOUNT_ID: &str = "bidder.eneftigo.testnet";

    /* buy_now */

    #[test]
    #[should_panic (expected = r#"This offering is Unstarted"#)]
    fn test_buy_now_too_early() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_buy(nft_account_id);
    }

    #[test]
    #[should_panic (expected = r#"This offering is Ended"#)]
    fn test_buy_now_too_late() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_buy(nft_account_id);
    }

    #[test]
    #[should_panic (expected = r#"Could not find NFT listing"#)]
    fn test_buy_now_nonexistent() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nonexistent_nft_account_id = AccountId::new_unchecked(NONEXISTENT_NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_buy(nonexistent_nft_account_id);
    }

    #[test]
    #[should_panic (expected = r#"Attached Near must be sufficient to pay the price of 1000 yocto Near"#)]
    fn test_buy_now_insufficient_deposit() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            999,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_buy(nft_account_id);
    }

    #[test]
    #[should_panic (expected = r#"You are late. All NFTs have been sold."#)]
    fn test_buy_now_while_running() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());

        marketplace.fpo_buy(nft_account_id.clone());
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id).expect("Could not get updated FPO");

        assert!(
            fpo.supply_left == 2,
            "Wrong supply_left"
        );
        assert!(
            !fpo.proposals.get(&1).unwrap().is_acceptable && fpo.proposals.get(&2).unwrap().is_acceptable && fpo.proposals.get(&3).unwrap().is_acceptable,
            "Proposals state incorrect"
        );
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![3, 2],
            "Proposals state incorrect"
        );

        marketplace.fpo_buy(nft_account_id.clone());
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id).expect("Could not get updated FPO");

        assert!(
            fpo.supply_left == 1,
            "Wrong supply_left"
        );
        assert!(
            !fpo.proposals.get(&1).unwrap().is_acceptable && fpo.proposals.get(&2).unwrap().is_acceptable && !fpo.proposals.get(&3).unwrap().is_acceptable,
            "Proposals state incorrect"
        );
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![2],
            "Proposals state incorrect"
        );

        marketplace.fpo_buy(nft_account_id.clone());
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id).expect("Could not get updated FPO");

        assert!(
            fpo.supply_left == 0,
            "Wrong supply_left"
        );
        assert!(
            !fpo.proposals.get(&1).unwrap().is_acceptable && !fpo.proposals.get(&2).unwrap().is_acceptable && !fpo.proposals.get(&3).unwrap().is_acceptable,
            "Proposals state incorrect"
        );
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![],
            "Proposals state incorrect"
        );

        marketplace.fpo_buy(nft_account_id.clone());

    }

    
    /* fpo_place_proposal */

    #[test]
    #[should_panic (expected = r#"This offering is Unstarted"#)]
    fn test_place_proposal_too_early() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            800,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_place_proposal(nft_account_id, 800);
    }

    #[test]
    #[should_panic (expected = r#"This offering is Ended"#)]
    fn test_place_proposal_too_late() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            800,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_place_proposal(nft_account_id, 800);
    }

    #[test]
    #[should_panic (expected = r#"Could not find NFT listing"#)]
    fn test_place_proposal_nonexistent() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 24).and_hms(00, 00, 00),
            800,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nonexistent_nft_account_id = AccountId::new_unchecked(NONEXISTENT_NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_place_proposal(nonexistent_nft_account_id, 800);
    }

    #[test]
    #[should_panic (expected = r#"Attached balance must be sufficient to pay the required deposit of 800 yocto Near"#)]
    fn test_place_proposal_insufficient_deposit() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            799,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_place_proposal(nft_account_id, 800);
    }

    #[test]
    #[should_panic(expected = r#"Proposed price is too low. The lowest acceptable price is 510"#)]
    fn test_place_proposal_outbid() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            500,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_place_proposal(nft_account_id, 500);
    }

    #[test]
    #[should_panic(expected = r#"Proposed price must be lower than buy now price of 1000"#)]
    fn test_place_proposal_above_buy_now() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1100,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_place_proposal(nft_account_id, 1100);
    }

    #[test]
    #[should_panic(expected = r#"Proposed price must be lower than buy now price of 1000"#)]
    fn test_place_proposal_at_buy_now() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        marketplace.fpo_place_proposal(nft_account_id, 1000);
    }

    #[test]
    fn test_place_proposal_successful() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo();
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        
        marketplace.fpo_place_proposal(nft_account_id.clone(), 550);
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id.clone()).expect("Could not get updated FPO");
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![4,3,2],
            "Wrong acceptable_proposals"
        );
        assert!(
            !fpo.proposals.get(&1).unwrap().is_acceptable &&
            fpo.proposals.get(&2).unwrap().is_acceptable &&
            fpo.proposals.get(&3).unwrap().is_acceptable &&
            fpo.proposals.get(&4).unwrap().is_acceptable,
            "Wrong acceptable_proposals"
        );
        assert!(
            fpo.next_proposal_id == 5,
            "Wrong next_proposal_id"
        );

        marketplace.fpo_place_proposal(nft_account_id.clone(), 950);
        let fpo = marketplace.fpos_by_contract_id.get(&nft_account_id.clone()).expect("Could not get updated FPO");
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![3,2,5],
            "Wrong acceptable_proposals"
        );
        assert!(
            !fpo.proposals.get(&1).unwrap().is_acceptable &&
            fpo.proposals.get(&2).unwrap().is_acceptable &&
            fpo.proposals.get(&3).unwrap().is_acceptable &&
            !fpo.proposals.get(&4).unwrap().is_acceptable &&
            fpo.proposals.get(&5).unwrap().is_acceptable,
            "Wrong acceptable_proposals"
        );
        assert!(
            fpo.next_proposal_id == 6,
            "Wrong next_proposal_id"
        );
    }

    /* fpo_place_proposal vs fpo_buy_now
    TODO: test interactions:
    - placing proposal after buy_now, check min accepted price is ok, check supply_left
    */


    /* Helpers */

    fn test_get_context(
        malicious: bool,
        datetime: DateTime<Utc>,
        attached_deposit: u128,
        storage_usage: u64,
    ) -> VMContext {
        let account_id = if malicious {
            AccountId::new_unchecked(MALICIOUS_ACCOUNT_ID.to_string())
        } else {
            AccountId::new_unchecked(BIDDER_ACCOUNT_ID.to_string())
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

    fn test_add_fpo(marketplace: &mut MarketplaceContract, fpo: &FixedPriceOffering) {
        marketplace
            .fpos_by_contract_id
            .insert(&fpo.nft_contract_id, fpo);
        let mut fpos_by_this_offeror = UnorderedSet::new(
            MarketplaceStorageKey::FposByOfferorIdInner {
                account_id_hash: hash_account_id(&fpo.offeror_id),
            }
            .try_to_vec()
            .unwrap(),
        );
        fpos_by_this_offeror.insert(&fpo.nft_contract_id.clone());
        marketplace
            .fpos_by_offeror_id
            .insert(&fpo.offeror_id, &fpos_by_this_offeror);
    }

    fn test_fpo() -> FixedPriceOffering {
        let nft_account_id = AccountId::new_unchecked(NFT_ACCOUNT_ID.to_string());
        let nft_account_id_hash = hash_account_id(&nft_account_id);
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());
        let start_timestamp = DateTime::parse_from_rfc3339("1975-05-26T00:00:00+00:00")
            .unwrap()
            .timestamp_nanos();
        let end_timestamp = DateTime::parse_from_rfc3339("1975-06-10T00:00:00+00:00")
            .unwrap()
            .timestamp_nanos();
        let fpo = FixedPriceOffering {
            nft_contract_id: nft_account_id.clone(),
            offeror_id: offeror_account_id.clone(),
            supply_total: 3,
            buy_now_price_yocto: 1000,
            min_proposal_price_yocto: Some(500),
            start_timestamp: Some(start_timestamp),
            end_timestamp: Some(end_timestamp),
            status: Unstarted,
            nft_metadata: nft_metadata(1),
            supply_left: 3,
            proposals: LookupMap::new(
                FixedPriceOfferingStorageKey::Proposals {
                    nft_contract_id_hash: nft_account_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            proposals_by_proposer: LookupMap::new(
                FixedPriceOfferingStorageKey::ProposalsByProposer {
                    nft_contract_id_hash: nft_account_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            acceptable_proposals: Vector::new(
                FixedPriceOfferingStorageKey::AcceptableProposals {
                    nft_contract_id_hash: nft_account_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            next_proposal_id: 4,
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
                is_acceptable: true,
            },
            FixedPriceOfferingProposal {
                id: 2,
                proposer_id: proposer2_id.clone(),
                price_yocto: 900,
                is_acceptable: true,
            },
            FixedPriceOfferingProposal {
                id: 3,
                proposer_id: proposer2_id.clone(),
                price_yocto: 700,
                is_acceptable: true,
            },
        ];
        for proposal in proposals_vec.iter() {
            fpo.proposals.insert(&proposal.id, &proposal);
        }
        fpo.acceptable_proposals.extend(vec![1, 3, 2]);

        let proposer1_id_hash = hash_account_id(&proposer1_id);
        let nft_account_id_hash = hash_account_id(&fpo.nft_contract_id);
        let mut proposals_by_proposer1: UnorderedSet<ProposalId> = UnorderedSet::new(
            FixedPriceOfferingStorageKey::ProposalsByProposerInner {
                nft_contract_id_hash: nft_account_id_hash,
                proposer_id_hash: proposer1_id_hash,
            }
            .try_to_vec()
            .unwrap(),
        );
        proposals_by_proposer1.extend(vec![1]);

        let proposer2_id_hash = hash_account_id(&proposer2_id);
        let mut proposals_by_proposer2: UnorderedSet<ProposalId> = UnorderedSet::new(
            FixedPriceOfferingStorageKey::ProposalsByProposerInner {
                nft_contract_id_hash: nft_account_id_hash,
                proposer_id_hash: proposer2_id_hash,
            }
            .try_to_vec()
            .unwrap(),
        );
        proposals_by_proposer2.extend(vec![2, 3]);

        fpo.proposals_by_proposer
            .insert(&proposer1_id, &proposals_by_proposer1);
        fpo.proposals_by_proposer
            .insert(&proposer2_id, &proposals_by_proposer2);
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