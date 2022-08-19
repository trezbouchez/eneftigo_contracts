use crate::*;
use near_sdk::{ext_contract, log, Gas, PromiseResult};

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);
const MIN_GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(100_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

pub trait NonFungibleTokenCore {
    //transfers an NFT to a receiver ID
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: NftId,
        approval_id: u64,
        memo: Option<String>,
    );

    //transfers an NFT to a receiver and calls a function on the receiver ID's contract
    /// Returns `true` if the token was transferred from the sender's account.
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: NftId,
        approval_id: u64,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;

    //get information about the NFT token passed in
    fn nft_token(&self, token_id: NftId) -> Option<JsonNft>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    //Method stored on the receiver contract that is called via cross contract call when nft_transfer_call is called
    /// Returns `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: NftId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    /*
        resolves the promise of the cross contract call to the receiver contract
        this is stored on THIS contract and is meant to analyze what happened in the cross contract call when nft_on_transfer was called
        as part of the nft_transfer_call method
    */
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,  // for logging event if we need to revert the transfer
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: NftId,
        previous_approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,           // this is for logging, too
    ) -> bool;
}

/*
    resolves the promise of the cross contract call to the receiver contract
    this is stored on THIS contract and is meant to analyze what happened in the cross contract call when nft_on_transfer was called
    as part of the nft_transfer_call method
*/ 
trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,  // for logging event if we need to revert the transfer
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: NftId,
        previous_approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,           // this is for logging, too
    ) -> bool;
}

#[near_bindgen]
impl NonFungibleTokenCore for NftContract {

    //implementation of the nft_transfer method. This transfers the NFT from the current owner to the receiver. 
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: NftId,
        approval_id: u64,
        memo: Option<String>,
    ) {
        //assert that the user attached exactly 1 yoctoNEAR. This is for security and so that the user will be redirected to the NEAR wallet. 
        assert_one_yocto();

        //get the sender to transfer the token from the sender to the receiver
        let sender_id = env::predecessor_account_id();

        //call the internal transfer method
        let previous_token = self.internal_transfer(
            &sender_id, 
            &receiver_id, 
            &token_id, 
            Some(approval_id),
            memo,
        );

        //we refund the owner for releasing the storage used up by the approved account IDs
        refund_approved_account_ids(
            previous_token.owner_id.clone(),
            &previous_token.approved_account_ids,
        );
    }

    //implementation of the transfer call method. This will transfer the NFT and call a method on the reciver_id contract
    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: NftId,
        approval_id: u64,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        // assert that exactly 1 yocto was attached (for security reasons). 
        assert_one_yocto();

        // get the GAS attached to the call
        let attached_gas = env::prepaid_gas();

        // make sure that the attached gas is greater than the minimum GAS for NFT transfer call.
        // to ensure that the cross contract call to nft_on_transfer won't cause a prepaid GAS error.
        // if this happens, the event will be logged in internal_transfer but the actual transfer logic will be
        // reverted due to the panic. This will result in the databases thinking the NFT belongs to the wrong person.
        assert!(
            attached_gas >= MIN_GAS_FOR_NFT_TRANSFER_CALL,
            "You cannot attach less than {:?} Gas to nft_transfer_call",
            MIN_GAS_FOR_NFT_TRANSFER_CALL
        );

        // get sender ID 
        let sender_id = env::predecessor_account_id();

        // transfer the token and get the previous token object
        let previous_token = self.internal_transfer(
            &sender_id, 
            &receiver_id, 
            &token_id, 
            Some(approval_id),
            memo.clone(),
        );

        // default the authorized_id to none
        let mut authorized_id = None; 
        // if the sender isn't the owner of the token, we set the authorized ID equal to the sender.
        if sender_id != previous_token.owner_id {
            authorized_id = Some(sender_id.to_string());
        }

        // initiating receiver's call and the callback
        ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id,
            previous_token.owner_id.clone(),    // could be sender_id, too (hadn't these two been equal we'd have panicked)
            token_id.clone(),
            msg,
            receiver_id.clone(), // contract account to make the call to
            NO_DEPOSIT, // attached deposit
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL, // attached GAS
        )
        // we then resolve the promise and call nft_resolve_transfer on our own contract
        .then(
            ext_self::nft_resolve_transfer(
                authorized_id,
                previous_token.owner_id,
                receiver_id,
                token_id,
                previous_token.approved_account_ids,
                memo,   // we introduce a memo for logging in the events standard
                env::current_account_id(), // contract account to make the call to
                NO_DEPOSIT, // attached deposit
                GAS_FOR_RESOLVE_TRANSFER, // attached GAS
            )
        )
        .into()
    }

    //get the information for a specific token ID
    fn nft_token(&self, token_id: NftId) -> Option<JsonNft> {
        //if there is some token ID in the tokens_by_id collection
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            //we'll get the metadata for that token
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            //we return the JsonNft (wrapped by Some since we return an option)
            Some(JsonNft {
                token_id,
                owner_id: token.owner_id,
                collection_id: token.collection_id,
                metadata,
                approved_account_ids: token.approved_account_ids,
                royalty: token.royalty,
            })
        } else { //if there wasn't a token ID in the tokens_by_id collection, we return None
            None
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for NftContract {
    // resolves the cross contract call when calling nft_on_transfer in the nft_transfer_call method
    // returns true if the token was successfully transferred to the receiver_id
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,  // for logging event if we need to revert the transfer
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: NftId,
        previous_approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,           // for logging, too
    ) -> bool {
         // whether receiver wants to return token back to the sender, based on `nft_on_transfer` result
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            // as per the standard, the nft_on_transfer should return whether we should return the token to its owner or not
            if let Ok(return_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                // if we don't need to return the token, we simply return true meaning everything went fine
                if !return_token {
                    // we refund the owner for releasing the storage used up by the approved account IDs
                    refund_approved_account_ids(previous_owner_id, &previous_approved_account_ids);
                    // since we've already transferred the token and nft_on_transfer returned false, we don't have to 
                    // revert the original transfer and thus we can just return true since nothing went wrong.
                    return true;
                }
            }
        }

        // get the new token object
        let mut new_token = if let Some(new_token) = self.tokens_by_id.get(&token_id) {
            if new_token.owner_id != receiver_id {
                // we refund the owner for releasing the storage used up by the approved account IDs
                refund_approved_account_ids(previous_owner_id, &previous_approved_account_ids);
                // the token is not owned by the receiver anymore. Can't return it.
                return true;
            }
            new_token
            // if there isn't a token object, it was burned and so we return true
        } else {
            // we could not get new token object (why?)
            refund_approved_account_ids(previous_owner_id, &previous_approved_account_ids);
            return true;
        };

        // if we haven't returned true yet that means that we should return the token to its original owner
        log!("Return {} from @{} to @{}", token_id, receiver_id, previous_owner_id);

        // remove the token from the receiver
        self.internal_remove_token_from_owner(&receiver_id, &token_id);
        
        // add the token to the original owner
        self.internal_add_token_to_owner(&previous_owner_id, &token_id);
        
        // change the token struct's owner to be the original owner 
        new_token.owner_id = previous_owner_id.clone();
        
        // refund the receiver for storage used up by any approved accounts they might have
        // added in the meantime
        refund_approved_account_ids(receiver_id.clone(), &new_token.approved_account_ids);

        // reset the approved account IDs to what they were before the transfer
        new_token.approved_account_ids = previous_approved_account_ids;

        // insert the token back into the tokens_by_id collection
        self.tokens_by_id.insert(&token_id, &new_token);

        // we need to log that the NFT was reverted back to the original owner.
        let nft_transfer_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id,  // optional authorized account ID to transfer the token on behalf of the old owner
                old_owner_id: receiver_id.to_string(),           // receiver is becoming the old owner on return    
                new_owner_id: previous_owner_id.to_string(),     // becoming new owner again
                token_ids: vec![token_id.to_string()],
                memo,
            }]),
        };
        env::log_str(&nft_transfer_log.to_string());

        false
    }
}