use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum Errors {
    // Parsing the origin file
    InvalidPath(String),
    FailedToOpen(String),
    InvalidJson(String),

    // Parsing a power
    InvalidPowerPath(String),
}

/// Contains information about a power.
#[derive(Debug, Clone, Deserialize)]
pub struct Power {
    name: Option<String>,
    description: Option<String>,
    hidden: Option<bool>,

    toggle: Option<Box<Power>>,
}

impl Power {
    /// Returns if the power is hidden
    pub fn is_hidden(&self) -> bool {
        self.hidden.unwrap_or(false)
    }
}

impl Default for Power {
    fn default() -> Self {
        Power {
            name: None,
            description: None,
            hidden: Some(true),
            toggle: None,
        }
    }
}

/// Contains information about an origin.
#[derive(Debug, Deserialize)]
pub struct Origin {
    name: String,
    description: String,
    powers: Option<Vec<String>>,
    impact: u8,
}

/// Parses the origins from the files.
pub fn parse_origins() -> Vec<Origin> {
    let mut origins = Vec::new();

    // Parses the origins from the files.
    for entry in glob::glob("./data/**/origins/*.json").expect("Pattern will never error.") {
        let result: Result<Origin, Errors> = entry.map_err(|e| Errors::InvalidPath(format!("{}", e)))
            .and_then(|path| {
                File::open(path.clone())
            }.map_err(|e| Errors::FailedToOpen(format!("Failed to open `{:?}`", path)))
                .and_then(|file| {
                    serde_json::from_reader(file).map_err(|e| Errors::InvalidJson(format!("{}", e)))
                }));

        match result {
            Err(error) => {
                eprintln!("Encountered errors while attempting to parse an origin. Skipping bad origin.\n{:?}", error);
                continue;
            }
            Ok(origin) => origins.push(origin)
        };
    }
    origins
}

/// Parses the powers from the given origins.
pub fn parse_powers(origins: &Vec<Origin>) -> HashMap<String, Power> {
    let mut parsed_powers = HashMap::new();

    for origin in origins {
        // If there are no powers then skip
        let Some(powers) = &origin.powers else {
            continue;
        };

        for power in powers {
            // Finds the power & parses it
            let result: Result<Power, Errors> = power_to_path(power).and_then(|path| {
                File::open(path.clone())
            }.map_err(|e| Errors::FailedToOpen(format!("Failed to open `{:?}`", path)))
                .and_then(|file| {
                    serde_json::from_reader(file).map_err(|e| Errors::InvalidJson(format!("{}", e)))
                }));

            // If there was an error, log it & then skip.
            let parsed_power = match result {
                Err(error) => {
                    eprintln!("Encountered errors while attempting to parse a power.\n\
                        Skipping bad power : {:?}\n\
                        From origin {}", error, origin.name);
                    continue;
                }
                Ok(power) => power
            };

            // Checks for nested toggle powers.
            if let Some(toggle) = parsed_power.toggle.clone() {
                parsed_powers.insert(format!("{power}_toggle"), *toggle);
            };

            parsed_powers.insert(power.clone(), parsed_power);
        }
    }

    parsed_powers
}

/// Gets the path of a power from origin data.
fn power_to_path(power: &String) -> Result<PathBuf, Errors> {
    let splits: Vec<&str> = power.split(":").collect();

    // The split must be of length two to be a valid path.
    let first = match splits.get(0) {
        None => { return Err(Errors::InvalidPowerPath(power.clone())); }
        Some(value) => { value }
    };
    let second = match splits.get(1) {
        None => { return Err(Errors::InvalidPowerPath(power.clone())); }
        Some(value) => { value }
    };

    // Creates the path to the power.
    let mut path_buf = PathBuf::new();
    path_buf.push(".");
    path_buf.push("data");
    path_buf.push(first);
    path_buf.push("powers");
    path_buf.push(second);
    path_buf.set_extension("json");
    Ok(path_buf)
}


/// Parses the powers & formats them & origins into markdown.
pub fn format_into_markdown(origins: &Vec<Origin>, powers: HashMap<String, Power>) -> (String, String) {
// Buffer to store the file data
    let mut data_prefix = "# Origins\n\n".to_string();
    let mut data = String::default();

    // Writes the origin & power data to the data buffer.
    for origin in origins {
        let name = origin.name.as_str();

        let impact = match origin.impact {
            2 => { "🟠🟠" }
            3 => { "🔴🔴🔴" } // Are red
            _ => { "🟢" }
        };

        // Adds the link to the origin to the start of the file.
        data_prefix.push_str(format!("- [{name}](#{})  \n", name.to_ascii_lowercase(), ).as_str());

        // Adds the header for the origin.
        data.push_str(format!("## {name}\n").as_str());

        // Adds the impact for the origin.
        data.push_str(format!("Impact : {impact}  \n").as_str());

        // Adds the description of the origin.
        data.push_str(format!("{}\n", origin.description.as_str()).as_str());

        // Adds the powers if they exist
        if let Some(origin_powers) = &origin.powers {
            data.push_str("#### Powers:\n");

            for power in origin_powers {
                let power = powers.get(&power.clone()).unwrap_or(&Power::default()).clone();

                // If the power is hidden then don't display it
                if power.is_hidden() { continue; }

                let name = power.name.unwrap_or("Missing name".to_string());
                let description = power.description.unwrap_or("Missing description".to_string());
                data.push_str(format!("**{}** : {}  \n", name, description).as_str())
            }
        }

        data.push_str("\n");
    }

    data_prefix.push_str("\n");
    (data_prefix, data)
}