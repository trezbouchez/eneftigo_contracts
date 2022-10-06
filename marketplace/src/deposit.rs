use crate::{constants::MIN_DEPOSIT, *};
use near_sdk::json_types::U128;

#[near_bindgen]
impl MarketplaceContract {
    #[payable]
    pub fn place_deposit(&mut self) {
        let account_id = env::predecessor_account_id();
        let deposited_amount_yocto = env::attached_deposit();

        let storage_before = env::storage_usage();

        // get current deposit for this account
        let current_deposit: Balance = self.storage_deposits.get(&account_id).unwrap_or(0);
        assert!(
            current_deposit + deposited_amount_yocto >= MIN_DEPOSIT,
            "Please deposit at least {}",
            MIN_DEPOSIT - current_deposit
        );
        let updated_deposit = current_deposit + deposited_amount_yocto;
        self.storage_deposits.insert(&account_id, &updated_deposit);

        let storage_after = env::storage_usage();
        let storage_cost = (storage_after - storage_before) as Balance * env::storage_byte_cost();
        if storage_cost > 0 {
            assert!(
                updated_deposit >= storage_cost,
                "Attached deposit too low. Could not deduct new customer deposit storage cost."
            );
            self.storage_deposits.insert(&account_id, &(updated_deposit - storage_cost));
        }
    }

    pub fn withdraw_deposit(&mut self, withdrawn_amount_yocto: U128) -> Promise {
        let account_id = env::predecessor_account_id();
        let withdrawn_balance: Balance = withdrawn_amount_yocto.0;
        // get current deposit for this account
        let current_deposit: Balance = self.storage_deposits.get(&account_id).unwrap_or(0);
        assert!(
            withdrawn_balance <= current_deposit,
            "Withdrawn amount exceeds your current deposit of {}",
            current_deposit
        );
        let updated_deposit = current_deposit - withdrawn_balance;
        self.storage_deposits.insert(&account_id, &updated_deposit);
        Promise::new(account_id).transfer(withdrawn_balance)
    }

    pub fn deposit(&self, account_id: AccountId) -> (U128, bool) {
        if let Some(current_deposit) = self.storage_deposits.get(&account_id) {
            (U128(current_deposit), true)
        } else {
            (U128(0), false)
        }
    }
}
