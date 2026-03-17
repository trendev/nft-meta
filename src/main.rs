use anyhow::{anyhow, Context, Result};
use borsh::BorshDeserialize;
use clap::{Parser, Subcommand};
use colored::Colorize;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

// ── Metaplex Token Metadata Program ID ──────────────────────────────────────
const METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

// ── Borsh structs — mirror the exact on-chain Metaplex binary layout ─────────

#[derive(BorshDeserialize, Debug)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

#[derive(BorshDeserialize, Debug)]
pub struct Data {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
}

/// We only deserialize the fields we need.
/// The trailing optional fields (edition_nonce, token_standard, collection…)
/// are intentionally omitted — borsh reads sequentially so this is safe.
#[derive(BorshDeserialize, Debug)]
pub struct NftMetadata {
    pub key: u8,                  // discriminator (4 = MetadataV1)
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub data: Data,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
}

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "nft-meta")]
#[command(about = "🔍 Fetch on-chain NFT metadata from Solana")]
#[command(version)]
struct Cli {
    /// Solana RPC endpoint [default: mainnet-beta]
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    rpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch metadata directly from a mint address
    Mint {
        /// NFT mint address (base58)
        address: String,
    },
    /// Auto-detect the mint from a transaction, then fetch its metadata
    Tx {
        /// Transaction signature (base58)
        signature: String,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client =
        RpcClient::new_with_commitment(cli.rpc_url.clone(), CommitmentConfig::confirmed());

    match &cli.command {
        Commands::Mint { address } => {
            let mint = Pubkey::from_str(address).context("Invalid mint address")?;
            fetch_and_print(&client, &mint)?;
        }

        Commands::Tx { signature } => {
            let sig = Signature::from_str(signature).context("Invalid transaction signature")?;
            println!("{}", "Scanning transaction accounts…".dimmed());
            let mint = find_mint_in_tx(&client, &sig)?;
            println!("{} {}\n", "Mint found:".dimmed(), mint.to_string().yellow());
            fetch_and_print(&client, &mint)?;
        }
    }

    Ok(())
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Derive the Metaplex metadata PDA for a given mint.
fn metadata_pda(mint: &Pubkey) -> Pubkey {
    let program_id = Pubkey::from_str(METADATA_PROGRAM_ID).unwrap();
    Pubkey::find_program_address(
        &[b"metadata", program_id.as_ref(), mint.as_ref()],
        &program_id,
    )
    .0
}

/// Walk every account in a transaction and return the first one
/// whose metadata PDA exists and is owned by the Metaplex program.
fn find_mint_in_tx(client: &RpcClient, sig: &Signature) -> Result<Pubkey> {
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    let tx = client
        .get_transaction_with_config(sig, config)
        .context("Failed to fetch transaction — check the signature and network")?;

    let versioned_tx = tx
        .transaction
        .transaction
        .decode()
        .ok_or_else(|| anyhow!("Could not decode the transaction payload"))?;

    let accounts = versioned_tx.message.static_account_keys();
    let metadata_program = Pubkey::from_str(METADATA_PROGRAM_ID)?;

    for account in accounts {
        let pda = metadata_pda(account);
        if let Ok(info) = client.get_account(&pda) {
            if info.owner == metadata_program {
                return Ok(*account);
            }
        }
    }

    Err(anyhow!(
        "No NFT mint found in this transaction.\n\
         Make sure you are passing a mint transaction signature."
    ))
}

/// Fetch the on-chain metadata account, deserialize it, and pretty-print it.
fn fetch_and_print(client: &RpcClient, mint: &Pubkey) -> Result<()> {
    let pda = metadata_pda(mint);

    let account = client
        .get_account(&pda)
        .context("No metadata account found — is this a valid NFT mint?")?;

    // The first byte is the Borsh discriminator; try_from_slice handles it as `key: u8`.
    let metadata = NftMetadata::try_from_slice(&account.data)
        .context("Failed to deserialize metadata")?;

    // Metaplex pads fixed-length string fields with \0 — strip them.
    let s = |raw: &str| raw.trim_matches('\0').trim().to_string();

    // ── Header ────────────────────────────────────────────────────────────────
    let divider = "━".repeat(50);
    println!("{}", divider.bold().green());
    println!("{}", " On-chain Metadata".bold().green());
    println!("{}", divider.bold().green());

    println!("  {:<22} {}", "Name:".cyan(),    s(&metadata.data.name).yellow());
    println!("  {:<22} {}", "Symbol:".cyan(),  s(&metadata.data.symbol).yellow());
    println!("  {:<22} {}", "URI:".cyan(),      s(&metadata.data.uri).yellow());
    println!(
        "  {:<22} {}",
        "Update Authority:".cyan(),
        metadata.update_authority.to_string().yellow()
    );
    println!(
        "  {:<22} {} bps  ({}%)",
        "Seller Fee:".cyan(),
        metadata.data.seller_fee_basis_points,
        metadata.data.seller_fee_basis_points / 100
    );
    println!(
        "  {:<22} {}",
        "Primary Sale:".cyan(),
        fmt_bool(metadata.primary_sale_happened)
    );
    println!(
        "  {:<22} {}",
        "Is Mutable:".cyan(),
        fmt_bool(metadata.is_mutable)
    );

    // ── Creators ──────────────────────────────────────────────────────────────
    println!("\n{}", " Creators".bold().green());
    println!("{}", divider.bold().green());

    match &metadata.data.creators {
        None => println!("  {}", "None".dimmed()),
        Some(creators) => {
            for (i, c) in creators.iter().enumerate() {
                println!("  {} #{}", "Creator".cyan(), i + 1);
                println!("    {:<14} {}", "Address:".cyan(),  c.address.to_string().yellow());
                println!("    {:<14} {}", "Verified:".cyan(), fmt_bool(c.verified));
                println!("    {:<14} {}%", "Share:".cyan(),   c.share);
            }
        }
    }

    println!("{}\n", divider.bold().green());
    Ok(())
}

fn fmt_bool(b: bool) -> colored::ColoredString {
    if b {
        "true".green()
    } else {
        "false".red()
    }
}
