
## Solana

### Run tests
cd solana/program && cargo test-bpf

### Deploy contract
cd solana/program && solana program deploy target/sbf-solana-solana/release/multisend.so --url mainnet-beta

### Init contract
cd solana/client && RPC_URL="https://api.mainnet-beta.solana.com" CONTRACT="4aNjeB2QaKoutwJ1GVmMZdXFKZxscHcCLMzqqfvek3XF" cargo run --bin update_settings

### Init bank
solana transfer ET6JPT6EEVQXtofs8Rvrb4Fo9wXFfLxiTBQvXPncXg4k 0.00089088 --url mainnet-beta --allow-unfunded-recipient