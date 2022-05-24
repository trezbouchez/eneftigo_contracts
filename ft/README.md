#1. Reset FT contract account
near delete ft.trez.testnet trez.testnet
near create-account ft.trez.testnet --masterAccount trez.testnet

#2. Build contract
yarn build

#3. Deploy contract
near deploy --wasmFile ./out/ft.wasm --accountId ft.trez.testnet

#4. Mint
near call ft.trez.testnet new '{
    "owner_id": "'ft.trez.testnet'", 
    "total_supply": "1000000000000000", 
    "metadata": { 
        "spec": "ft-1.0.0", 
        "name": "ENEFTIGO", 
        "symbol": "TIGO", 
        "decimals": 8 }
    }' --accountId ft.trez.testnet

#5. Transfer storage deposit
near call ft.trez.testnet storage_deposit '' --accountId hubi.testnet --amount 0.00125

#6. Transfer tokens
near call ft.trez.testnet ft_transfer '{"receiver_id": "hubi.testnet", "amount": "10000000000"}' --accountId ft.trez.testnet --amount 0.000000000000000000000001       

