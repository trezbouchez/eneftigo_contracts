#[cfg(test)]
mod seller_tests {
    use crate::external::NftMetadata;
    use crate::internal::{hash_account_id, hash_offering_id};
    use crate::FixedPriceOffering;
    use crate::FixedPriceOfferingProposal;
    use crate::FixedPriceOfferingStatus::*;
    use crate::FixedPriceOfferingStorageKey;
    use crate::{MarketplaceContract, MarketplaceStorageKey};
    use crate::{NftCollectionId, OfferingId, ProposalId};
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::collections::{LookupMap, TreeMap, UnorderedSet, Vector};
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, VMContext};

    const MARKETPLACE_ACCOUNT_ID: &str = "marketplace.eneftigo.testnet";
    const NFT_CONTRACT_ID: &str = "nft.eneftigo.testnet";
    const NONEXISTENT_NFT_CONTRACT_ID: &str = "nonexistent.eneftigo.testnet";
    const OFFEROR_ACCOUNT_ID: &str = "offeror.eneftigo.testnet";
    // const MALICIOUS_ACCOUNT_ID: &str = "malicious.eneftigo.testnet";
    const PROPOSER1_ACCOUNT_ID: &str = "proposer1.eneftigo.testnet";
    const PROPOSER2_ACCOUNT_ID: &str = "proposer2.eneftigo.testnet";
    const BIDDER_ACCOUNT_ID: &str = "bidder.eneftigo.testnet";

    /* buy_now */

    #[test]
    #[should_panic(expected = r#"This offering is Unstarted"#)]
    fn test_buy_now_too_early() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_buy(nft_contract_id, collection_id);
    }

    #[test]
    #[should_panic(expected = r#"This offering is Ended"#)]
    fn test_buy_now_too_late() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_buy(nft_contract_id, collection_id);
    }

    #[test]
    #[should_panic(expected = r#"Could not find NFT listing"#)]
    fn test_buy_now_nonexistent() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nonexistent_nft_contract_id =
            AccountId::new_unchecked(NONEXISTENT_NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_buy(nonexistent_nft_contract_id, collection_id);
    }

    #[test]
    #[should_panic(
        expected = r#"Attached Near must be sufficient to pay the price of 1000 yocto Near"#
    )]
    fn test_buy_now_insufficient_deposit() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            999,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_buy(nft_contract_id, collection_id);
    }

    #[test]
    fn test_buy_now_while_running() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");

        assert!(fpo.supply_left == 2, "Wrong supply_left");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(proposals == vec![3, 2], "Proposals state incorrect");

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");

        assert!(fpo.supply_left == 1, "Wrong supply_left");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(proposals == vec![2], "Proposals state incorrect");

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");

        assert!(fpo.supply_left == 0, "Wrong supply_left");
        assert!(
            fpo.proposals.to_vec().is_empty(),
            "Proposals state incorrect"
        );
    }

    #[test]
    #[should_panic(expected = r#"This offering is Ended"#)]
    fn test_buy_now_when_no_supply() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace.fpos_by_id.get(&offering_id).unwrap();
        assert!(fpo.supply_left == 2, "supply_left incorrect");

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace.fpos_by_id.get(&offering_id).unwrap();
        assert!(fpo.supply_left == 1, "supply_left incorrect");

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace.fpos_by_id.get(&offering_id).unwrap();
        assert!(fpo.supply_left == 0, "supply_left incorrect");

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
    }

    /* fpo_place_proposal */

    #[test]
    #[should_panic(expected = r#"This offering is Unstarted"#)]
    fn test_place_proposal_too_early() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            800,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_place_proposal(nft_contract_id, collection_id, U128(800));
    }

    #[test]
    #[should_panic(expected = r#"This offering is Ended"#)]
    fn test_place_proposal_too_late() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            800,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_place_proposal(nft_contract_id, collection_id, U128(800));
    }

    #[test]
    #[should_panic(expected = r#"Could not find NFT listing"#)]
    fn test_place_proposal_nonexistent() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 24).and_hms(00, 00, 00),
            800,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nonexistent_nft_contract_id =
            AccountId::new_unchecked(NONEXISTENT_NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nonexistent_nft_contract_id, collection_id, U128(800));
    }

    #[test]
    #[should_panic(
        expected = r#"Attached balance must be sufficient to pay the required deposit of 800 yocto Near"#
    )]
    fn test_place_proposal_insufficient_deposit() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            799,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id, collection_id, U128(800));
    }

    #[test]
    #[should_panic(expected = r#"Proposed price is too low. The lowest acceptable price is 510"#)]
    fn test_place_proposal_outbid() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            500,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id, collection_id, U128(500));
    }

    #[test]
    #[should_panic(expected = r#"Proposed price must be lower than buy now price of 1000"#)]
    fn test_place_proposal_above_buy_now() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1100,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id, collection_id, U128(1100));
    }

    #[test]
    #[should_panic(expected = r#"Proposed price must be lower than buy now price of 1000"#)]
    fn test_place_proposal_at_buy_now() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id, collection_id, U128(1000));
    }

    #[test]
    fn test_place_proposal_successful() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(550));
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();

        assert!(proposals == vec![4, 3, 2], "Wrong acceptable_proposals");
        assert!(fpo.next_proposal_id == 5, "Wrong next_proposal_id");

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(950));
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();

        assert!(proposals == vec![3, 2, 5], "Wrong acceptable_proposals");
        assert!(fpo.next_proposal_id == 6, "Wrong next_proposal_id");
    }

    /* fpo_place_proposal vs fpo_buy_now */

    #[test]
    #[should_panic(expected = r#"Proposed price is too low. The lowest acceptable price is 710"#)]
    fn test_place_too_low_proposal_after_buy_now() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(
            proposals == vec![3, 2],
            "acceptable_proposals not updated on buy_now"
        );
        assert!(fpo.supply_left == 2, "Supply left not updated");

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(700));
    }

    #[test]
    fn test_place_acceptable_proposal_after_buy_now() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_buy(nft_contract_id.clone(), collection_id);
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(
            proposals == vec![3, 2],
            "acceptable_proposals not updated on buy_now"
        );
        assert!(fpo.supply_left == 2, "Supply left not updated");

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(800));
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(
            proposals == vec![4, 2],
            "acceptable_proposals not updated on buy_now"
        );
    }

    #[test]
    #[should_panic(expected = r#"Proposals are not accepted for this offering"#)]
    fn test_place_proposal_for_buy_now_only() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let fpo = test_fpo(false);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(800));
    }

    /* fpo_modify_proposal */

    #[test]
    #[should_panic(expected = r#"This offering is Unstarted"#)]
    fn test_modify_proposal_too_early() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_modify_proposal(nft_contract_id, collection_id, 1, U128(850));
    }

    #[test]
    #[should_panic(expected = r#"This offering is Ended"#)]
    fn test_modify_proposal_too_late() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_modify_proposal(nft_contract_id, collection_id, 1, U128(900));
    }

    #[test]
    #[should_panic(expected = r#"No prior proposal from this account"#)]
    fn test_modify_proposal_unauthorized_user_1() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        marketplace.fpo_modify_proposal(nft_contract_id.clone(), collection_id, 2, U128(950));
    }

    #[test]
    #[should_panic(
        expected = r#"Proposal with ID 2 from account bidder.eneftigo.testnet not found"#
    )]
    fn test_modify_proposal_unauthorized_user_2() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(800));
        marketplace.fpo_modify_proposal(nft_contract_id.clone(), collection_id, 2, U128(950));
    }
    #[test]
    #[should_panic(expected = r#"Price must be an integer multple of 10 yocto Near"#)]
    fn test_modify_proposal_price_not_multiple_of_step() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let fpo = test_fpo(true);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(805));
    }

    #[test]
    #[should_panic(
        expected = r#"Attached balance must be sufficient to pay the required deposit supplement of 350 yocto Near"#
    )]
    fn test_modify_proposal_insufficient_deposit() {
        let context = test_get_context(
            PROPOSER1_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            10,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_modify_proposal(nft_contract_id.clone(), collection_id, 1, U128(850));
    }
    #[test]
    fn test_modify_proposal_price_increase() {
        let context = test_get_context(
            PROPOSER1_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            350,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_modify_proposal(nft_contract_id.clone(), collection_id, 1, U128(850));
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        assert!(
            fpo.proposals.get(1).unwrap().price_yocto == 850,
            "Price has not been updated"
        );
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(proposals == vec![3, 1, 2], "Wrong acceptable_proposals");
        assert!(fpo.supply_left == 3, "supply_left incorrect");
    }

    #[test]
    fn test_modify_proposal_buy_now() {
        let context = test_get_context(
            PROPOSER1_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            500,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_modify_proposal(nft_contract_id.clone(), collection_id, 1, U128(1000));
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(proposals == vec![3, 2], "Wrong acceptable_proposals");
        assert!(fpo.supply_left == 2, "supply_left incorrect");
    }

    /* fpo_revoke_proposal */

    #[test]
    #[should_panic(expected = r#"This offering is Unstarted"#)]
    fn test_revoke_proposal_too_early() {
        let context = test_get_context(
            OFFEROR_ACCOUNT_ID,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_revoke_proposal(nft_contract_id, collection_id, 1);
    }

    #[test]
    #[should_panic(expected = r#"This offering is Ended"#)]
    fn test_revoke_proposal_too_late() {
        let context = test_get_context(
            OFFEROR_ACCOUNT_ID,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_revoke_proposal(nft_contract_id, collection_id, 1);
    }

    #[test]
    #[should_panic(expected = r#"No prior proposal from this account"#)]
    fn test_revoke_proposal_unauthorized_user() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        marketplace.fpo_revoke_proposal(nft_contract_id.clone(), collection_id, 2);
    }

    #[test]
    #[should_panic(
        expected = r#"Proposal with ID 2 from account bidder.eneftigo.testnet not found"#
    )]
    fn test_revoke_proposal_unauthorized_user_2() {
        let context = test_get_context(
            BIDDER_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;

        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(800));
        marketplace.fpo_revoke_proposal(nft_contract_id.clone(), collection_id, 2);
    }
    #[test]
    #[should_panic(expected = r#"This proposal has been outbid. The deposit has been returned"#)]
    fn test_revoke_outbid_proposal() {
        let context = test_get_context(
            PROPOSER1_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        marketplace.fpo_place_proposal(nft_contract_id.clone(), collection_id, U128(800));
        marketplace.fpo_revoke_proposal(nft_contract_id.clone(), collection_id, 1);
    }

    #[test]
    fn test_revoke_the_only_proposal_success() {
        let context = test_get_context(
            PROPOSER1_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_revoke_proposal(nft_contract_id.clone(), collection_id, 1);
        let fpo = marketplace.fpos_by_id.get(&offering_id).unwrap();
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
        assert!(proposals == vec![3, 2], "acceptable_proposals incorrect");
    }

    #[test]
    fn test_revoke_one_of_many_proposals_success() {
        let context = test_get_context(
            PROPOSER2_ACCOUNT_ID,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            1000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(true);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        marketplace.fpo_revoke_proposal(nft_contract_id.clone(), collection_id, 2);
        let fpo = marketplace.fpos_by_id.get(&offering_id).unwrap();
        let proposals: Vec<ProposalId> = fpo
            .proposals
            .to_vec()
            .iter()
            .map(|proposal| proposal.id)
            .collect();
       assert!(proposals == vec![1, 3], "acceptable_proposals incorrect");
    }

    /* Helpers */

    fn test_get_context(
        account_id_str: &str,
        datetime: DateTime<Utc>,
        attached_deposit: u128,
        storage_usage: u64,
    ) -> VMContext {
        let account_id = AccountId::new_unchecked(account_id_str.to_string());
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
        marketplace.fpos_by_id.insert(&fpo.offering_id, fpo);
        let mut fpos_by_this_offeror = UnorderedSet::new(
            MarketplaceStorageKey::FposByOfferorIdInner {
                account_id_hash: hash_account_id(&fpo.offeror_id),
            }
            .try_to_vec()
            .unwrap(),
        );
        fpos_by_this_offeror.insert(&fpo.offering_id);
        marketplace
            .fpos_by_offeror_id
            .insert(&fpo.offeror_id, &fpos_by_this_offeror);
    }

    fn test_fpo(allow_proposals: bool) -> FixedPriceOffering {
        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: NftCollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        let offering_id_hash = hash_offering_id(&offering_id);
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());
        let start_timestamp = DateTime::parse_from_rfc3339("1975-05-26T00:00:00+00:00")
            .unwrap()
            .timestamp_nanos();
        let end_timestamp = DateTime::parse_from_rfc3339("1975-06-10T00:00:00+00:00")
            .unwrap()
            .timestamp_nanos();
        let nft_metadata = NftMetadata::new(
            &String::from("Bored Grapes"),
            &String::from("https://ipfs.io/ipfs/QmcRD4wkPPi6dig81r5sLj9Zm1gDCL4zgpEj9CfuRrGbzF"),
        );
        let fpo = FixedPriceOffering {
            offering_id,
            offeror_id: offeror_account_id.clone(),
            nft_metadata,
            supply_total: 3,
            buy_now_price_yocto: 1000,
            min_proposal_price_yocto: if allow_proposals { Some(500) } else { None },
            start_timestamp: Some(start_timestamp),
            end_timestamp: Some(end_timestamp),
            status: Unstarted,
            // nft_metadata: nft_metadata(1),
            supply_left: 3,
            proposals: Vector::new(
                FixedPriceOfferingStorageKey::Proposals { offering_id_hash }
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
            },
            FixedPriceOfferingProposal {
                id: 2,
                proposer_id: proposer2_id.clone(),
                price_yocto: 900,
            },
            FixedPriceOfferingProposal {
                id: 3,
                proposer_id: proposer2_id.clone(),
                price_yocto: 700,
            },
        ];
        for proposal in proposals_vec.iter() {
            fpo.proposals.push(&proposal);
        }
        fpo.sort_proposals();
    }

    // fn nft_metadata(index: i32) -> TokenMetadata {
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
}
