# nft-meta

A Rust CLI that **reads** on-chain NFT metadata from the [Metaplex Token Metadata](https://developers.metaplex.com/token-metadata) program on Solana.

> **This tool is a read-only inspector.** It does not mint, update, or manage NFTs.
> The NFTs it reads are created and owned by third parties through the Metaplex standard — `nft-meta` simply decodes and displays their on-chain metadata.

## Install

```bash
cargo install --path .
```

## Usage

> The CLI targets **Solana Mainnet** by default — no extra flag needed.

### From a mint address
```bash
cargo run -- mint <ADDRESS>
```

### From a transaction signature
```bash
cargo run -- tx <SIGNATURE>
```
The CLI will scan all accounts in the transaction, auto-detect the NFT mint,
and print its full metadata.

### Choosing a cluster

Use `--cluster` (or `-c`) to select a Solana network:

| Value          | RPC endpoint                              |
|----------------|-------------------------------------------|
| `mainnet` (default) | `https://api.mainnet-beta.solana.com` |
| `testnet`      | `https://api.testnet.solana.com`          |
| `devnet`       | `https://api.devnet.solana.com`           |
| `localhost`    | `http://localhost:8899`                   |
| Any URL        | Used as-is                                |

```bash
# Devnet
cargo run -- -c devnet mint <ADDRESS>

# Custom RPC
cargo run -- -c https://your-rpc-endpoint.com mint <ADDRESS>
```

> ⚠️ Note the `--` separator between `cargo run` and your CLI arguments.
> Everything after `--` is passed to your program, not to Cargo.

### Getting help

```bash
cargo run -- --help          # general help
cargo run -- mint --help     # mint subcommand help
cargo run -- tx --help       # tx subcommand help
```

## Output example

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 On-chain Metadata
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Mint:                  DtRy2fCC7GGir4TMMEcTowA4FgMqgu74BXrjnU8MGMb7
  Name:                  Lot 11 ...
  Symbol:                S
  URI:                   https://soulboundimages.s3.eu-west-3.amazonaws.com/a/mmq8nk3smq26om.j
  Update Authority:      HdknM9vFE15udMbju6CqisFthkN6yZE7kZvCc1...
  Seller Fee:            0 bps  (0%)
  Primary Sale:          false
  Is Mutable:            true
  Edition Nonce:         252
  Token Standard:        ProgrammableNonFungible

 Collection
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  None

 Uses
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  None

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
