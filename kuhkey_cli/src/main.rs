use anyhow::bail;
use clap::{Parser, Subcommand};
use rand::{
    SeedableRng,
    distr::{Alphanumeric, Distribution},
    rngs::ChaCha12Rng,
};

const DEFAULT_PASSWORD_LENGTH: usize = 16;
pub const MINIMUM_PASSWORD_LENGTH: usize = 8;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new KuhKey Vault
    Init,

    /// Unlock your vault
    Unlock,

    /// Add a password:
    /// Identifier (email, username, domain)
    /// ?Password?
    Add {
        identifier: String,

        #[arg(long)]
        gen_p: bool,
    },

    /// Generate a new password
    ///
    /// This accepts an optional length which when set
    /// is the length of the password
    ///
    /// If this is not set, the default password will be `16` characters long
    Gen {
        #[arg(long = "len")]
        length: Option<usize>,
    },

    /// Get the password for a given identifier
    ///
    /// An optional `--copy` puts the password into your clipboard
    ///
    /// `./kuhkey_cli get --copy github`
    ///
    /// Retrives the password for github and puts it into your clipboard
    Get {
        /// Identifier for the password entry
        identifier: String,

        /// Copy the password directly to the clipboard
        #[arg(long)]
        copy: bool,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Init => {
            println!("Init kuh key")
        }
        Commands::Unlock => {
            println!("Unlocking kuh vault")
        }
        Commands::Add { identifier, gen_p } => {
            println!("{identifier}");

            if *gen_p {
                println!("Generating password for you..");
                todo!()
            }
        }
        Commands::Gen { length } => {
            let p_length = length.unwrap_or(DEFAULT_PASSWORD_LENGTH);

            if p_length < MINIMUM_PASSWORD_LENGTH {
                bail!(
                    "InvalidPassword - Expected a minimum password length of {MINIMUM_PASSWORD_LENGTH}, found {p_length}"
                )
            }

            println!("Generating a password with length {p_length}");
            let password = generate_password(p_length);

            println!("Password: {}", password);
        }
        Commands::Get { identifier, copy } => {
            println!(
                "Searching password for {} - Copy to clipboard: {copy}",
                identifier
            );
        }
    }

    Ok(())
}

/// TODO: MOVE THIS INTO kuhkey_core
// From: https://rust-random.github.io/book/guide-rngs.html#state-and-seeding
// It is worth noting that a CSPRNG’s security relies absolutely on being seeded with a secure random key.
// Should the key be known or guessable, all output of the CSPRNG is easy to guess.
// This implies that the seed should come from a trusted source; usually either the OS or another CSPRNG.
// For this purpose, we recommend using the getrandom crate which interfaces the OS’s secure random interface.
fn generate_random_seed() -> Result<[u8; 32], getrandom::Error> {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf)?;
    Ok(buf)
}

/// TODO: MOVE THIS INTO kuhkey_core
///
/// Generates a random password
fn generate_password(length: usize) -> String {
    let Ok(seed) = generate_random_seed() else {
        // Failed to generate a random seed?!
        // Probably crash?
        panic!("Failed to generate random seed");
    };

    let mut rng = ChaCha12Rng::from_seed(seed);
    let rng_str: String = Alphanumeric
        .sample_iter(&mut rng)
        .take(length)
        .map(char::from)
        .collect();

    rng_str
}
