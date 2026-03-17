# nft-meta

A Rust CLI to fetch on-chain NFT metadata from Solana — fully decoded, no truncation.

## Install

```bash
cargo install --path .
```

## Usage

> The CLI targets **Solana Mainnet** by default — no extra flag needed.

### From a mint address
```bash
cargo run -- mint DtRy2fCC7GGir4TMMEcTowA4FgMqgu74BXrjnU8MGMb7
```

### From a transaction signature
```bash
cargo run -- tx 5KtPn3...yourTxSignature...
```
The CLI will scan all accounts in the transaction, auto-detect the NFT mint,
and print its full metadata.

### Using a custom RPC (optional but recommended for rate limits)

**Inline:**
```bash
SOLANA_RPC_URL=https://your-rpc-endpoint.com cargo run -- mint <ADDRESS>
```

**Export once, run many times:**
```bash
export SOLANA_RPC_URL=https://your-rpc-endpoint.com
cargo run -- mint <ADDRESS>
cargo run -- tx <SIGNATURE>
```

**Via `--rpc-url` flag:**
```bash
cargo run -- --rpc-url https://your-rpc-endpoint.com mint <ADDRESS>
```

> ⚠️ Note the `--` separator between `cargo run` and your CLI arguments.
> Everything after `--` is passed to your program, not to Cargo.

## Output example

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 On-chain Metadata
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Name:                  Lot 11 — Soulbound #42
  Symbol:                S
  URI:                   https://soulboundimages.s3.eu-west-3.amazonaws.com/a/mmq8nk3smq26om.json
  Update Authority:      HdknM9vFE15udMbju6CqisFthkN6yZE7kZvCc1...
  Seller Fee:            0 bps  (0%)
  Primary Sale:          false
  Is Mutable:            true

 Creators
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Creator #1
    Address:       HdknM9vFE15udMbju6CqisFthkN6yZE7kZvCc1...
    Verified:      true
    Share:         100%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## How it works

- **`mint` command** — derives the Metaplex metadata PDA from the mint address,
  fetches the raw account data, and deserializes it via Borsh.
- **`tx` command** — fetches the transaction, walks every account key,
  checks if a Metaplex metadata PDA exists for it, and uses the first match as the mint.

The Borsh structs are defined manually to match the exact on-chain Metaplex layout,
with no dependency on the Metaplex SDK.
