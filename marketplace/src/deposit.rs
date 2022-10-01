use crate::{
    *,
    constants::{MIN_DEPOSIT},
};

#[near_bindgen]
impl MarketplaceContract {
    #[payable]
    pub fn place_deposit(&mut self) {
        let account_id = env::predecessor_account_id();
        let deposited_amount_yocto = env::attached_deposit();

        // get current deposit for this account
        let current_deposit: Balance = self.storage_deposits.get(&account_id).unwrap_or(0);
        assert!(
            current_deposit + deposited_amount_yocto >= MIN_DEPOSIT,
            "Please deposit at least {}",
            MIN_DEPOSIT - current_deposit
        );
        self.storage_deposits
            .insert(&account_id, &(current_deposit + deposited_amount_yocto));
    }

    pub fn withdraw_deposit(&mut self, withdrawn_amount_yocto: Balance) {
        let account_id = env::predecessor_account_id();

        // get current deposit for this account
        let current_deposit: Balance = self.storage_deposits.get(&account_id).unwrap_or(0);
        assert!(
            withdrawn_amount_yocto <= current_deposit,
            "Withdrawn amount exceeds your current deposit of {}",
            current_deposit
        );
        self.storage_deposits
            .insert(&account_id, &(current_deposit - withdrawn_amount_yocto));
        Promise::new(account_id).transfer(withdrawn_amount_yocto);
    }

    pub fn deposit(&self, account_id: AccountId) -> (Balance, bool) {
        if let Some(current_deposit) = self.storage_deposits.get(&account_id) {
            (current_deposit, true)
        } else {
            (0, false)
        }
    }
}
