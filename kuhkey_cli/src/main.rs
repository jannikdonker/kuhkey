use std::{env, ffi::OsString, io::IsTerminal};

use anyhow::bail;
use clap::{Parser, Subcommand};
use cli_clipboard::ClipboardProvider;
use colored::Colorize;
use kuhkey_core::generator::{PasswordOptions, generate_password};
use kuhkey_ttt::{History, lookup_location};

const DEFAULT_PASSWORD_LENGTH: usize = 16;
pub const MINIMUM_PASSWORD_LENGTH: usize = 1;
const ENVIRONMENT_VARIABLE_NAME: &str = "KUHKEY_CLI_KEY";

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
    Add {
        identifier: String,

        #[arg(long = "generate", short = 'g')]
        generate: bool,
    },

    /// Generate a new password
    Gen {
        /// Put the password into the users clipboard after generating
        #[arg(long = "copy", short = 'c')]
        copy: bool,

        /// The length of the password
        #[arg(long = "len")]
        length: Option<usize>,

        /// Include uppercase characters
        #[arg(long = "upper", short = 'u')]
        uppercase: bool,

        /// Include lowercase characters
        #[arg(long = "lower", short = 'l')]
        lowercase: bool,

        /// Include numbers
        #[arg(long = "numbers", short = 'n')]
        numbers: bool,

        /// Include symbols
        #[arg(long = "symbols", short = 's')]
        symbols: bool,

        /// Exclude look alikes
        #[arg(long = "no-alikes")]
        exclude_lookalikes: bool,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Init => {
            println!("Init kuh key")
        }
        Commands::Unlock => cli_unlock()?,
        Commands::Add {
            identifier,
            generate,
        } => cli_add_entry(identifier, *generate)?,
        Commands::Gen {
            copy,
            length,
            uppercase,
            lowercase,
            numbers,
            symbols,
            exclude_lookalikes,
        } => {
            let length = length.unwrap_or(DEFAULT_PASSWORD_LENGTH);

            cli_gen_password(
                *copy,
                length,
                *uppercase,
                *lowercase,
                *numbers,
                *symbols,
                *exclude_lookalikes,
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

/// Unlock the kuh key vault by placing an environment variable in the shells process
///
/// This function first checks, whether or not the user ran this command directly:
///
/// `./kuhkey_cli unlock`
///
/// If yes, we bail early because we can not create such an env variable:
///
/// By running the command directly, the shell starts a child process where any env var we created
///
/// gets destroyed as soon as the command is done executing.
///
/// Since we need the env var to stick around as long as the user uses the shell, we need to create it another way:
///
/// `eval $(...)`
///
/// This runs the `unlock` command as the shell itself and places the env var in the shells memory
///
/// Other functions that require the vault to be unlocked can now check for this env var to be sure
///
/// the user is authenticated
///
/// The environment variable gets destroyed as soon as the user closes the shell
fn cli_unlock() -> anyhow::Result<()> {
    // User ran `kuhkey_cli unlock` directly
    // => Exit the application
    if std::io::stdout().is_terminal() {
        eprintln!(
            "Warning: You ran `kuhkey_cli unlock` directly and the session can not be saved."
        );
        eprintln!("Run this instead to unlock your current shell: \n");
        // TODO: Obviously temporary fix :)
        eprintln!("   eval $(cargo r --bin kuhkey_cli unlock)");
        bail!("{}", "Ran unlock command from terminal".red());
    }

    //TODO: Password/Key logic
    eprintln!("TODO: Implement password/key logic");

    // only after the master psw was right
    cli_location_check()?;

    let session_key = String::from("KUH_KEY_SESSION_KEY");
    println!("export {}=\"{}\"", ENVIRONMENT_VARIABLE_NAME, session_key);

    eprintln!("KuhKey Vault unlocked, welcome back :)");
    Ok(())
}

// checks where the login comes from, new country => 2fa
// everything goes to eprintln! bc unlocks stdout gets evaled by the shell
fn cli_location_check() -> anyhow::Result<()> {
    // offline or ipinfo is down we just skip it
    let (Ok(location), Ok(mut history)) = (lookup_location(), History::load()) else {
        eprintln!("{}", "Warning: location check skipped".yellow());
        return Ok(());
    };

    if history.needs_2fa(&location.country) {
        eprintln!("New login country: {}", location.country);
        eprintln!("{}", "2FA required".red());
        // TODO: ask for the totp code here (totp-rs) and only go on if it was right
        bail!("2FA is not implemented yet");
    }

    // only record accepted logins, a failed one from russia shouldnt add russia to the list
    history.record(&location);
    if history.save().is_err() {
        eprintln!("{}", "Warning: could not save login history".yellow());
    }

    Ok(())
}

/// Checks if the kuhkey env var exists
///
/// If yes, it contains the value of the env var
fn check_unlocked() -> Option<OsString> {
    env::var_os(ENVIRONMENT_VARIABLE_NAME)
}

// The generic parameter is necessary as T will be a WindowsClipboard on Windows or a MacClipboard on Mac...
fn cli_gen_password(
    copy: bool,
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
    exclude_lookalikes: bool,
) -> Result<(), anyhow::Error> {
    if length < MINIMUM_PASSWORD_LENGTH {
        let err = format!("InvalidPassword - Expected a minimum password length of {MINIMUM_PASSWORD_LENGTH}, found {length}").red();
        bail!(err)
    }

    // Has the user supplied any options?
    let user_options = [uppercase, lowercase, numbers, symbols, exclude_lookalikes]
        .iter()
        .any(|&b| b);

    // TODO: Figure out how to clear the clipboard later
    let clipboard = cli_clipboard::ClipboardContext::new().ok();

    let options = if user_options {
        PasswordOptions {
            length,
            uppercase: uppercase,
            lowercase: lowercase,
            numbers: numbers,
            symbols: symbols,
            exclude_lookalikes: exclude_lookalikes,
        }
    } else {
        PasswordOptions {
            length,
            ..Default::default()
        }
    };

    let password = generate_password(&options);

    let generated_password = match password {
        Ok(p) => p,
        Err(err) => {
            println!("{:?}", err);
            bail!("Unable to generate password");
        }
    };

    if copy && clipboard.is_some() {
        // Unwrap here is safe because we know from the check above that clipboard is some
        #[cfg(not(target_os = "linux"))]
        let res = clipboard.unwrap().set_contents(generated_password.clone());

        #[cfg(target_os = "linux")]
        let res = copy_to_clipboard(&generated_password);

        match res {
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
        // Check if we landed here because the clipboard failed
        if copy {
            println!("It looks like KuhKey was unable to put the password into your clipboard");
        }

        println!("Your password: \n{}", generated_password)
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    // Try Wayland first, then X11 fallbacks
    let candidates: &[(&str, &[&str])] = &[
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];

    for (bin, args) in candidates {
        let Ok(mut child) = Command::new(bin)
            .args(*args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue; // binary not installed, try next
        };

        match child.stdin.take() {
            Some(mut s) => s.write_all(text.as_bytes())?,
            None => bail!("Could not write to child stdin"),
        }

        // Don't wait(): the tool forks/stays alive to serve the selection
        return Ok(());
    }
    anyhow::bail!("No clipboard tool found (install wl-clipboard, xclip or xsel)")
}

fn cli_add_entry(_identifier: &str, _generate_password: bool) -> anyhow::Result<()> {
    let Some(key) = check_unlocked() else {
        bail!(
            "{}",
            "KuhKey Vault not unlocked! Run `./kuhkey_cli unlock` first!".red()
        )
    };

    println!("Key of env var: {key:?}");

    Ok(())
}
#[cfg(test)]
mod tests {
    use crate::{Cli, cli_gen_password};
    use clap::CommandFactory;

    #[test]
    fn cli_args_are_valid() {
        // should catch clap config mistakes (dup flags etc) without running anything
        Cli::command().debug_assert();
    }

    #[test]
    fn cli_gen_succeeds() {
        assert!(cli_gen_password(false, 16, false, false, true, true, false).is_ok());
    }

    #[test]
    fn cli_gen_zero_length_fails() {
        assert!(cli_gen_password(false, 0, false, false, false, false, false).is_err());
    }

    #[test]
    fn cli_gen_no_upper_and_no_lower_fails() {
        assert!(cli_gen_password(false, 16, false, false, false, false, false).is_ok());
    }

    #[test]
    fn cli_gen_too_short_for_classes_fails() {
        assert!(cli_gen_password(false, 1, false, false, false, false, false).is_err());
    }

    #[test]
    fn cli_gen_password_length_one() {
        assert!(cli_gen_password(false, 1, true, false, false, false, false).is_ok());
    }
}
