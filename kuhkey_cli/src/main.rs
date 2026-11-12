use anyhow::bail;
use clap::{Parser, Subcommand};
use colored::Colorize;
use kuhkey_core::generator::{PasswordOptions, generate_password};

const DEFAULT_PASSWORD_LENGTH: usize = 16;
pub const MINIMUM_PASSWORD_LENGTH: usize = 1;

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
    Gen {
        /// Put the password into the users clipboard after generating
        #[arg(long = "copy")]
        copy: bool,

        /// The length of the password
        #[arg(long = "len")]
        length: Option<usize>,

        /// Don't include upper case characters
        #[arg(long = "no-upper")]
        no_uppercase: bool,

        /// Dont't include lower case characters
        #[arg(long = "no-lower")]
        no_lowercase: bool,

        /// Include numbers
        #[arg(long = "numbers")]
        numbers: bool,

        /// Include symbols
        #[arg(long = "symbols")]
        symbols: bool,

        /// Exclude look alikes
        #[arg(long = "no-alikes")]
        exclude_lookalikes: bool,

        /// Includes each of the flags above
        /// Also ignores any other flags except the `no-upper` / `no-lower` ones
        #[arg(long = "all")]
        all_params: bool,
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
                println!("Generating password for you...");
                todo!()
            }
        }
        Commands::Gen {
            copy,
            length,
            no_uppercase,
            no_lowercase,
            numbers,
            symbols,
            exclude_lookalikes,
            all_params,
        } => {
            let length = length.unwrap_or(DEFAULT_PASSWORD_LENGTH);

            cli_gen_password(
                *copy,
                length,
                *no_uppercase,
                *no_lowercase,
                *numbers,
                *symbols,
                *exclude_lookalikes,
                *all_params,
            )?
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

fn cli_gen_password(
    copy: bool,
    length: usize,
    no_uppercase: bool,
    no_lowercase: bool,
    numbers: bool,
    symbols: bool,
    exclude_lookalikes: bool,
    all_params: bool,
) -> Result<(), anyhow::Error> {
    if length < MINIMUM_PASSWORD_LENGTH {
        let err = format!("InvalidPassword - Expected a minimum password length of {MINIMUM_PASSWORD_LENGTH}, found {length}").red();
        bail!(err)
    }

    if no_lowercase && no_uppercase {
        let err = "Can not set `no-upper` and `no-lower` flag at the same time".red();
        bail!("{err}");
    }

    let mut options = PasswordOptions {
        length,
        // When --no-upper / --no-lower is set, we need to tell the generator, NOT to use them
        // --no-uper (true) => uppercase (false)
        uppercase: !no_uppercase,
        lowercase: !no_lowercase,
        numbers: numbers,
        symbols: symbols,
        exclude_lookalikes: exclude_lookalikes,
    };

    if all_params {
        options.exclude_lookalikes = false;
        options.numbers = true;
        options.symbols = true;
    }

    let password = generate_password(&options);

    let Ok(generated_password) = password else {
        bail!("Unable to generate password");
    };

    if copy {
        match terminal_clipboard::set_string(generated_password.clone()) {
            Ok(_) => {
                println!("{}", "The password is in your clipboard".green());
            }
            Err(_) => {
                println!(
                  "{}",
                  "Failed to put the password into your clipboard! Maybe try again, or without the `--copy` flag"
                      .red()
              )
            }
        }
    } else {
        println!("Your password: \n{}", generated_password)
    }

    Ok(())
}
