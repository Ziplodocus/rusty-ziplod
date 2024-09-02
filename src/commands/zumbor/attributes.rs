use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Attribute {
    Charisma,
    Strength,
    Wisdom,
    Agility,
}

impl Attribute {
    pub const VALUES: [Attribute; 4] = [
        Attribute::Charisma,
        Attribute::Strength,
        Attribute::Wisdom,
        Attribute::Agility,
    ];
}

impl Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Attribute::Charisma => write!(f, "Charisma"),
            Attribute::Strength => write!(f, "Strength"),
            Attribute::Wisdom => write!(f, "Wisdom"),
            Attribute::Agility => write!(f, "Agility"),
        }
    }
}

impl From<Attribute> for String {
    fn from(value: Attribute) -> Self {
        match value {
            Attribute::Charisma => "Charisma".to_string(),
            Attribute::Strength => "Strength".to_string(),
            Attribute::Wisdom => "Wisdom".to_string(),
            Attribute::Agility => "Agility".to_string(),
        }
    }
}

impl TryFrom<String> for Attribute {
    type Error = Error;
    fn try_from(key: String) -> Result<Attribute, Error> {
        match key.as_str() {
            "Charisma" | "charisma" => Ok(Attribute::Charisma),
            "Strength" | "strength" => Ok(Attribute::Strength),
            "Agility" | "agility" => Ok(Attribute::Agility),
            "Wisdom" | "wisdom" => Ok(Attribute::Wisdom),
            _ => Err(Error::Plain("Not a key m8")),
        }
    }
}

impl TryFrom<&str> for Attribute {
    type Error = Error;
    fn try_from(key: &str) -> Result<Attribute, Error> {
        match key {
            "Charisma" | "charisma" => Ok(Attribute::Charisma),
            "Strength" | "strength" => Ok(Attribute::Strength),
            "Agility" | "agility" => Ok(Attribute::Agility),
            "Wisdom" | "wisdom" => Ok(Attribute::Wisdom),
            _ => Err(Error::Plain("Not a key m8")),
        }
    }
}
