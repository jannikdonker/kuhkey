use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::{Result, bail};
use kuhkey_core::{APP_ID, current_timestamp};
use serde::{Deserialize, Serialize};

// smaller then this and we will just observe 
const LEARNING_LOGINS: u32 = 10;
// we have to limit it --> huge file otherwise 
const MAX_HISTORY: usize = 100;

// what we get back from ipinfo
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub ip: String,
    #[serde(default)]
    pub city: String,
    // z.B. "DE"
    pub country: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub ip: String,
    pub country: String,
    pub city: String,
    pub timestamp: u64,
}

// locations.json
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct History {
    // logins per country, e.g. {"DE": 12, "CZ": 2}
    pub countries: BTreeMap<String, u32>,
    // last logins, newest at the end
    pub logins: Vec<Login>,
}

// ~/.local/share/de.edah.kuhkey/locations.json on linux
fn history_path() -> Result<PathBuf> {
    let Some(dir) = dirs::data_dir() else {
        bail!("could not find the data dir");
    };
    Ok(dir.join(APP_ID).join("locations.json"))
}

impl History {
    pub fn load() -> Result<Self> {
        let path = history_path()?;
        // no file yet => first login ever
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = history_path()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    // only looks at the country, city lookups may be too inaccurate
    pub fn needs_2fa(&self, country: &str) -> bool {
        let total: u32 = self.countries.values().sum();
        // still learning => let the user in
        if total < LEARNING_LOGINS {
            return false;
        }
        !self.countries.contains_key(country)
    }

    // only call this after the login got accepted
    pub fn record(&mut self, location: &Location) {
        *self.countries.entry(location.country.clone()).or_insert(0) += 1;

        self.logins.push(Login {
            ip: location.ip.clone(),
            country: location.country.clone(),
            city: location.city.clone(),
            timestamp: current_timestamp(),
        });

        // we add one at a time, so dropping the oldest one is enough
        if self.logins.len() > MAX_HISTORY {
            self.logins.remove(0);
        }
    }
}

// ask ipinfo who we are, locally we dont know our public ip
// fails if theres no country (e.g. private ip)
#[cfg(not(target_arch = "wasm32"))]
pub fn lookup_location() -> Result<Location> {
    let location = ureq::get("https://ipinfo.io/json")
        .config()
        // dont let unlock hang forever on a bad connection
        .timeout_global(Some(std::time::Duration::from_secs(5)))
        .build()
        .call()?
        .body_mut()
        .read_json()?;
    Ok(location)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(country: &str) -> Location {
        Location {
            ip: String::from("1.2.3.4"),
            city: String::from("Berlin"),
            country: country.to_owned(),
        }
    }

    // history with `n` logins from `country`
    fn history_with(country: &str, n: u32) -> History {
        let mut h = History::default();
        for _ in 0..n {
            h.record(&location(country));
        }
        h
    }

    #[test]
    fn new_country_while_learning_is_ok() {
        let h = history_with("DE", LEARNING_LOGINS - 1);
        assert!(!h.needs_2fa("RU"));
    }

    #[test]
    fn known_country_is_ok() {
        let h = history_with("DE", LEARNING_LOGINS);
        assert!(!h.needs_2fa("DE"));
    }

    #[test]
    fn new_country_needs_2fa() {
        let h = history_with("DE", LEARNING_LOGINS);
        assert!(h.needs_2fa("RU"));
    }

    #[test]
    fn record_counts_per_country() {
        let mut h = history_with("DE", 3);
        h.record(&location("CZ"));
        assert_eq!(h.countries.get("DE"), Some(&3));
        assert_eq!(h.countries.get("CZ"), Some(&1));
    }

    #[test]
    fn login_list_is_capped() {
        let h = history_with("DE", MAX_HISTORY as u32 + 20);
        assert_eq!(h.logins.len(), MAX_HISTORY);
        // the counter keeps counting tho
        assert_eq!(h.countries.get("DE"), Some(&(MAX_HISTORY as u32 + 20)));
    }

    #[test]
    fn ipinfo_json_parses() {
        let json = r#"{"ip":"1.2.3.4","city":"Berlin","region":"Berlin","country":"DE","loc":"52.5,13.4"}"#;
        let l: Location = serde_json::from_str(json).unwrap();
        assert_eq!(l, location("DE"));
    }

    #[test]
    fn ipinfo_json_without_country_fails() {
        let l = serde_json::from_str::<Location>(r#"{"ip":"10.0.0.1","bogon":true}"#);
        assert!(l.is_err());
    }
}
