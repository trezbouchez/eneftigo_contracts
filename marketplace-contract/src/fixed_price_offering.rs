const STORAGE_PER_FIXED_PRICE_OFFERING: u128 = 1000 * STORAGE_PRICE_PER_BYTE;   // TODO: adjust

// contains fixed-price offering parameters
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FixedPriceOffering {
    pub offeror_id: AccountId,
    // pub approval_id: u64, // market contract's approval ID to transfer the token on behalf of the owner
    
    // the NFT contract that's deployed and initialized; no tokens have been minted yet
    pub nft_contract_id: String,    
    //actual token ID for sale
    pub token_id: String,
    //sale price in yoctoNEAR that the token is listed for
    pub sale_conditions: SalePriceInYoctoNear,
}

#[near_bindgen]
impl Contract {

    pub fn fixed_price_offering_total_supply() -> U128 {
        U128(self.fixed_price_offerings_by_contract_id.len() as u128)
    }

    pub fn fixed_price_offering_add(
        &mut self,
        nft_contract_id: AccountId,
        offering: FixedPriceOffering,
    ) {
    // make sure it's called by marketplace 
    assert_eq!(
        &env::predecessor_account_id(),
        &owner_id,
        "Only Eneftigo marketplace owner can add offering."
    );

    // make sure we have enough NEAR for storage (dot 0 converts from U128 to u128)
    // let storage_required = (self.fixed_price_offering_total_supply().0 + 1) as u128 * STORAGE_PER_FIXED_PRICE_OFFERING;
    
    //create the unique sale ID which is the contract + DELIMITER + token ID
    let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
    
    //insert the key value pair into the sales map. Key is the unique ID. value is the sale object
    self.fixed_price_offerings_by_contract_id(

    )
    self.sales.insert(
        &contract_,
        &Sale {
            owner_id: owner_id.clone(), //owner of the sale / token
            approval_id, //approval ID for that token that was given to the market
            nft_contract_id: nft_contract_id.to_string(), //NFT contract the token was minted on
            token_id: token_id.clone(), //the actual token ID
            sale_conditions, //the sale conditions 
       },
    );

    //Extra functionality that populates collections necessary for the view calls 

    //get the sales by owner ID for the given owner. If there are none, we create a new empty set
    let mut by_owner_id = self.by_owner_id.get(&owner_id).unwrap_or_else(|| {
        UnorderedSet::new(
            StorageKey::ByOwnerIdInner {
                //we get a new unique prefix for the collection by hashing the owner
                account_id_hash: hash_account_id(&owner_id),
            }
            .try_to_vec()
            .unwrap(),
        )
    });
    
    //insert the unique sale ID into the set
    by_owner_id.insert(&contract_and_token_id);
    //insert that set back into the collection for the owner
    self.by_owner_id.insert(&owner_id, &by_owner_id);

    //get the token IDs for the given nft contract ID. If there are none, we create a new empty set
    let mut by_nft_contract_id = self
        .by_nft_contract_id
        .get(&nft_contract_id)
        .unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ByNFTContractIdInner {
                    //we get a new unique prefix for the collection by hashing the owner
                    account_id_hash: hash_account_id(&nft_contract_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });
    
    //insert the token ID into the set
    by_nft_contract_id.insert(&token_id);
    //insert the set back into the collection for the given nft contract ID
    self.by_nft_contract_id
        .insert(&nft_contract_id, &by_nft_contract_id);
}
// place a fixed-price offering. The sale will go through as long as your deposit is greater than or equal to the list price
#[payable]
pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String) {
    //get the attached deposit and make sure it's greater than 0
    let deposit = env::attached_deposit();
    assert!(deposit > 0, "Attached deposit must be greater than 0");

    //convert the nft_contract_id from a AccountId to an AccountId
    let contract_id: AccountId = nft_contract_id.into();
    //get the unique sale ID (contract + DELIMITER + token ID)
    let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
    
    //get the sale object from the unique sale ID. If the sale doesn't exist, panic.
    let sale = self.sales.get(&contract_and_token_id).expect("No sale");
    
    //get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
    let buyer_id = env::predecessor_account_id();
    assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");
    
    //get the u128 price of the token (dot 0 converts from U128 to u128)
    let price = sale.sale_conditions.0;

    //make sure the deposit is greater than the price
    assert!(deposit >= price, "Attached deposit must be greater than or equal to the current price: {:?}", price);

    //process the purchase (which will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties) 
    self.process_purchase(
        contract_id,
        token_id,
        U128(deposit),
        buyer_id,
    );
}