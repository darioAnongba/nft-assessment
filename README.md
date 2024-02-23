# Open Source RGB/Lightning Developer assessment test

Using rgb-lib, develop a small NFT minting service that takes a definition of an NFT token and a blinded UTXO where the newly minted assets are to be sent.
A command-line interface is sufficient, a regtest setup is preferable and you can use any of the rgb-lib supported languages: rust, python, kotlin or swift.

A demo of how rgb-lib can be use is available here: https://github.com/RGB-Tools/rgb-lib-python/tree/master/demo

For general knowledge on the RGB protocol, you can consult the website: https://rgb.info/

## Run process

Follow these instructions to run the app. The process mints an NFT by sending a POST request to the endpoint `/rgb/assets/issue`. The server will mint the NFT as a RGB-21 token and send the assets to the blinded UTXO in the request.

### Infrastructure initialization

The environments is set up using a Makefile and Docker-compose. _Tested on MacOS M2_.

Bring up the infrastructure:
```
make up
```

Fill the `.env` with a mnemonic seed. The rest of the config, located under `/config` can be changed but should work out of the box. To change it, create a file `/config/development.toml|yaml|json` and override the params you want:
```
# .env
ASSESSMENT_RGB__MNEMONIC="YOUR MNEMONIC SEED PHRASE HERE"
```
> Note that a new wallet will be created under `/storage/rgblib` for every mnemonic phrase used.


And finally start the application (you can run multiple applications to test multiple senders/receivers by changing the port in the config `config/default.toml` and the mnemonic in `.env`. Examples use `3000` for sender and `3001` for receiver)
```
cargo run
```

Once the app has successfully started, get an address from the wallet by querying the server at ``/rgb/wallet/address``:
```
curl --location 'localhost:3000/rgb/wallet/address'
```


And fund the wallet with some BTC
```
make send wallet=miner recipient=YOUR_BTC_ADDRESS_HERE amount=100
```

### Mint NFTs

The application is essentially an Axum HTTP server exposed on the port defined in the config (default 3000), accessible at `localhost:3000`. It is easy to query it by using cURL or Postman.

#### Prepare UTXOs

In RGB, we need special UTXOs to hold `RGB allocations` where our assets will be allocated. By default 5 UTXOs will be created. To create these UTXOs, run:

```
curl --location 'localhost:3000/rgb/wallet/prepare-issuance' \
--header 'Content-Type: application/json' \
--data '{}'
```

#### Mint NFT

To mint and NFT and send it to a specific address, send a POST request with the desired body to the endpoint `/rgb/assets/issue`.

The predominant nomenclature, given by the Ethereum community are Fungible (ERC-20) and Non-Fungible (ERC-21) Tokens. This nomenclature has been changed in RGB to NIA (Non Inflatable Asset), UDA (Unique Digital Asset) and CFA (Collectible Fungible Assets). CFAs are thus not NFTs by definition. As asked, we will then issue an UDA. All types of assets can be issued though by changing the "asset_type" value in the request.

To avoid having to upload files, the `filename` param is taken from the files defined in the `media_dir` field of the config, (`storage/media` by default). Just place the file you want inside this folder.

As stated by the assignment, the user only inputs a blinded UTXO. We will simply post the consignment to the proxy server defined in the config under `proxy_server_url`:

> To get a receiver blinded UTXO, simply run the app with a different mnemonic on port 3001 and call `localhost:3001/rgb/assets/invoice`, then get the `recipient_id`. Be sure to have available allocations.

```
curl --location 'localhost:3000/rgb/assets/issue' \
--header 'Content-Type: application/json' \
--data '{
    "asset_type": "UDA",
    "name": "Bitcoin Whitepaper NFT",
    "ticker": "BWT",
    "details": "The Bitcoin whitepaper as a unique NFT",
    "filename": "bitcoin.pdf",
    "recipient": "{recipient_id}"
}'
```

The query returns the `asset_id`, which can then be used to get the asset by calling `/rgb/assets`:
```
curl --location 'localhost:3000/rgb/assets'
```

And mine some blocks:
```
make mine wallet=miner blocks=4
```

On the recipient end, refresh the assets:
```
curl --location --request POST 'localhost:3001/rgb/assets/refresh'
```

Do the same on the sender:
```
curl --location --request POST 'localhost:3000/rgb/assets/refresh'
```

## Shutdown
Run 
```
make down
```
when you are done.

## Code structure explanation

The code is implemented using the clean architecture design pattern, more specifically the hexagonal clean architecture variant.

Without going into much detail. Clean architecture separates the codebase in independent layers and maximazes separation of concerns. The hexagonal variant uses domains that encapsulate the logic of a specific feature of the application (like users, lightning, rgb, etc.). This codebase lacks a "data layer" and a "business layer" because there is no storage (no models) and the business logic is implemented directly inside the handlers (which is not a good practice, but okay for such a small app).

Clean architecture extensively uses traits, allowing switching implementations, only affecting the adapters layer. Traits also allow for easier testing by mocking.

The structure is like this:

```
.
├── adapters      // External components and dependencies
│   ├── app     
│   ├── axum    
│   ├── config  
│   ├── logging 
│   └── rgb     
├── application   // Exposed API of the application
│   ├── dtos      // Application requests and responses
│   └── errors    // Application errors
└── domains       // Domains of the application
    └── rgb       // RGB domain, only domain in this app
        ├── api   
        ├── entities 
```

