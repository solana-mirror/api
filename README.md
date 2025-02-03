# Solana Mirror 
API that parses associated token accounts (ATAs), transactions and generates chart data for a Solana wallet address. As of now, there are no dApp implementations -- the balances are fetched from the wallet itself.

## Endpoints

### GET `/balances/<address>?showApps=`

Gets the wallet's ATAs, parsed with their respective metadata and balances

#### Response example

```json
[{
    "mint": "",
    "ata": "",
    "coingeckoId": "",
    "decimals": 0,
    "name": "",
    "symbol": "",
    "image": "",
    "price": 0,
    "balance": {
        "amount": 0,
        "formatted": 0
    }
}]
```

- `mint`: mint adddress of the token
- `ata`: address of the associated token account of the specified wallet for the specified mint
- `coingeckoId`: coingecko ID of the mint
- `decimals`: decimals of the mint 
- `name`: the Metaplex name of the mint 
- `symbol`: the Metaplex symbol of the mint
- `image`: the `image` propriety from the mint's IPFS link provided by Metaplex 
- `price`: the Jupiter price against USDC of the mint
- `balance`: the balance of the ATA

### GET `/transactions/<address>?index=`

Gets the wallet's transactions and parses the balances before and after the tx of the tokens that were transferred

#### Query params
- Index: only fetch a range of the signatures. Index should be formatted like `start_idx-end_idx` (`0-1`)

#### Response example

```json
{
    "transactions": [{
        "blockTime": 0,
        "signatures": [""],
        "logs" [""],
        "balances": {
            "So11111111111111111111111111111111111111112": {
                "pre": {
                    "amount": 0,
                    "formatted": 0
                },
                "post": {
                    "amount": 0,
                    "formatted": 0
                }
            }
        },
        "parsedInstructions": [""]
    }],
    "count": 1
}
```

- `blockTime`: UNIX timestamp of the transactions
- `signatures`: signatures of the transactions
- `logs`: transaction logs
- `balances`: map with all the tokens that have changed in balance for the specified address in the transaction, with the balance before (`pre`) and after (`post`) the transaction
- `parsedInstructions`: an array created based on parsing the logs with the `instruction` keyword

### GET `/chart/<address>/<timeframe>`

Gets the wallet's transactions and builds state with all the balances at each interval.

- Timeframe format example: `7d` (for seven days)
- Timeframe ranges supported: daily (`d`), hourly (`h`). Note: the limit for hourly resolution is 90 days (or 2160 hours)

#### Response example 

```json
[{
    "timestamp": 0,
    "balances": {
        "So11111111111111111111111111111111111111112": {
            "amount": 0,
            "formatted": 0,
            "usdValue": 0
        }
    },
    "usdValue": 0
}]
```

- `timestamp`: the UNIX timestamp of the state represented
- `balances`: a map with all the balances of the wallet at the given timestamp
- `usdValue`: the cumulative usdValue of the wallet 


## Status codes
- 400: The provided address (or timeframe) are not valid
- 5xx: Internal error
