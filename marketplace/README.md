#1. Reset NFT marketplace account
near delete marketplace.trez.testnet trez.testnet
near create-account marketplace.trez.testnet --masterAccount trez.testnet

#2. Build contract
yarn build

#3. Deploy contract
near deploy --wasmFile ./out/marketplace.wasm --accountId marketplace.trez.testnet

#4. Initialize contract
near call marketplace.trez.testnet new '{"owner_id": "marketplace.trez.testnet"}' --accountId marketplace.trez.testnet 

#5. Add Fixed-Price Offering listing
near call marketplace.trez.testnet fpo_list '{
    "nft_contract_id": "nft.trez.testnet", 
    "offeror_id": "hubi.testnet",
    "supply_total": 3,
    "buy_now_price_yocto": 1000,
    "min_proposal_price_yocto": 100,
    "nft_metadata":  {
        "title": "Nie bedzie niczego!", 
        "description": "Krzychu", 
        "media": "https://eneftigo.s3.eu-central-1.amazonaws.com/konon-kononowicz-bestia.gif"
    }
}' --accountId marketplace.trez.testnet --amount 0.1

#6. Show FPO listings
near view marketplace.trez.testnet fpos '{
    "from_index": "0",
    "limit": 2
}'

#7. Make unacceptable proposals 
near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 90
}' --accountId marketplace.trez.testnet --depositYocto 9

near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 100
}' --accountId marketplace.trez.testnet --depositYocto 9

#8. Make acceptable proposal
near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 100
}' --accountId marketplace.trez.testnet --depositYocto 10

#9. View proposals
near view marketplace.trez.testnet fpo_proposals '{
    "nft_contract_id": "nft.trez.testnet",
    "limit": 10
}'

#10. Make more proposals
near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 110
}' --accountId marketplace.trez.testnet --depositYocto 11

near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 130
}' --accountId marketplace.trez.testnet --depositYocto 13

#11. Make proposal that's supposed to be outbid
near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 100
}' --accountId marketplace.trez.testnet --depositYocto 10

#12. Make acceptable proposal
near call marketplace.trez.testnet fpo_place_proposal '{
    "nft_contract_id": "nft.trez.testnet",
    "proposed_price_yocto": 120
}' --accountId marketplace.trez.testnet --depositYocto 12

----------------------

0. Offering party prepares an offer including:
- `media`
- `offer_type` (`fixed_price`, `auction`, other? - list is extensible)
- `nft_type` (`vanilla`, `music_rights`)
- (for fixed_price): `price`, (for auction: `initial_price`, `min_sale_value`)
- `total_supply` offered
- `sale_start` (can be now)
- `sale_end` (optional if type is fixed_price)
- `all_or_nothing` flag (only if total_supply > 1 & sale_end is set)
- optional: NFT icon
- optional: NFT name
- optional: NFT symbol

all_or_nothing, if true means:
- for fixed_price the buyers are not actually buying but merely subscribing and leaving deposits; when the period ends the NFT are minted if buyers subscribed for total_supply, otherwise deposits are returned
- for auction buyers will only get their NFTs if the number of offers was >= total_supply

1. NFT contract gets deployed to a new address

2. It gets initialized with proper metadata which includes:
- owner, which is marketplace
- total_supply
- name
- symbol
- icon
- reference includes all legal shit and stuff

3. Marketplace accountID gets approval (to mint and transfer)

(NO NFT are minted yet)

# Q1: Would it make sense to include all primary offer functionality into the NFT contract itself? 

>>> `fixed_price` sale

3. media (and potentially other stuff) gets stored in IPFS
4. Sale gets added to contract by calling list_primary with parameters including
- NFT contract account ID
- price
- total_supply
- sale_start
- sale_end (if any)
- all_or_nothing
- media URL
5. Front-end lists such offer so we'll need enumeration methods
6. If between dates, users are able to make offers or buy (depending on all_or_nothing flag state)
7. If user buys or sale ends and the conditions are met the contract:
- mints NFT
- removes sale
- how about archiving? it's all in the blockchain, isn't it? should we use events for bids?
- how do we notify users? think about it later. backend can be listening for blockchain events and send pushes


We need a database of all NFT contracts. This should be back-end. However, how about storing the list in the market contract itself? Those which were successfull, at least.











No FTS are not minted yet and the contract is not even initialized ()
New sale is created, we need this data:
- seller ID
- seller crypto account (can it be create on-the-fly?, can we maintain an account on their behalf?)
- 