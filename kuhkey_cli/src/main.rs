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
            todo!()

            // println!("Password: {}", password);
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
