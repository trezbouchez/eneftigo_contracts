/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

// const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "ENEFTIGO".to_string(),
                symbol: "TIGO".to_string(),
                icon: Some("data:image/png;base64,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".to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), total_supply.into());
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
