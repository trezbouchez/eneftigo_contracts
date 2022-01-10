#1. Reset NFT contract account
near delete nft.trez.testnet trez.testnet
near create-account nft.trez.testnet --masterAccount trez.testnet

#2. Build contract
yarn build

#3. Deploy contract
near deploy --wasmFile ../out/main.wasm --accountId nft.trez.testnet

#4. Initialize contract
near call nft.trez.testnet new_default_meta '{"owner_id": "nft.trez.testnet"}' --accountId nft.trez.testnet 

#5. Mint
near call nft.trez.testnet nft_mint '{
    "token_id": "konon1", 
    "metadata": {
        "title": "Nie bedzie niczego!", 
        "description": "Krzychu", 
        "media": "https://eneftigo.s3.eu-central-1.amazonaws.com/konon-kononowicz-bestia.gif"
    }, 
    "receiver_id": "hubi.testnet",
    "perpetual_royalties": {
        "trez.testnet": 2000      
    }
}' --accountId nft.trez.testnet --amount 0.1

near call nft.trez.testnet nft_mint '{
    "token_id": "konon2", 
    "metadata": {
        "title": "Piwo bezalkoholowe!", 
        "description": "Krzychu", 
        "media": "https://bafybeif7gnlxkeuppgw2usl2asi5hzyqn6uctsyqr7vgdrqtg77ia4l4sq.ipfs.dweb.link/"
    }, 
    "receiver_id": "patka.testnet",
    "perpetual_royalties": {
        "trez.testnet": 2000      
    }
}' --accountId nft.trez.testnet --amount 0.1

#NOTE: May need logging in as hubi.testnet and patka.testnet

#6. Transfer hubi->patka
near call nft.trez.testnet nft_transfer '{
    "receiver_id": "patka.testnet", 
    "token_id": "konon1", 
    "approval_id": 0,
    "memo": "Bierz Krzysia"
}' --accountId hubi.testnet --depositYocto 1

#7. Approval - patka approves trez.testnet to transfer konon1
near call nft.trez.testnet nft_approve '{
    "token_id": "konon1", 
    "account_id": "trez.testnet"
}' --accountId patka.testnet --deposit 0.1

#8. View patka's tokens and check trez.testnet has been approved
near view nft.trez.testnet nft_tokens_for_owner '{"account_id": "patka.testnet", "limit": 10}'

#9. Approved trez.testnet transfers konon1 to hubi.testnet
near call nft.trez.testnet nft_transfer '{
    "receiver_id": "hubi.testnet",
    "token_id": "konon1",
    "approval_id": 0,
    "memo": "Masz go z powrotem!"
}' --accountId trez.testnet --depositYocto 1

#10. Check approved accounts are empty now
near view nft.trez.testnet nft_tokens_for_owner '{"account_id": "hubi.testnet", "limit": 10}'

