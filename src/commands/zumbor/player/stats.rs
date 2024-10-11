use serde::{Deserialize, Serialize};
use serenity::all::{ActionRow, ActionRowComponent};

use crate::{commands::zumbor::attributes::Attribute, errors::Error};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Stats {
    pub charisma: i16,
    pub strength: i16,
    pub wisdom: i16,
    pub agility: i16,
}

impl Stats {
    pub fn builder() -> StatsBuilder {
        StatsBuilder::default()
    }

    pub fn get(&self, key: Attribute) -> i16 {
        match key {
            Attribute::Charisma => self.charisma,
            Attribute::Strength => self.strength,
            Attribute::Wisdom => self.wisdom,
            Attribute::Agility => self.agility,
        }
    }

    pub fn get_mut(&mut self, key: Attribute) -> &mut i16 {
        match key {
            Attribute::Charisma => &mut self.charisma,
            Attribute::Strength => &mut self.strength,
            Attribute::Wisdom => &mut self.wisdom,
            Attribute::Agility => &mut self.agility,
        }
    }

    pub fn sum(&self) -> i16 {
        self.agility + self.charisma + self.strength + self.wisdom
    }

    pub fn get_max(&self) -> i16 {
        [self.agility, self.charisma, self.strength, self.wisdom]
            .into_iter()
            .reduce(|acc, nu| if nu > acc { nu } else { acc })
            .expect("Array is not empty?")
    }
}

impl TryFrom<Vec<ActionRow>> for Stats {
    type Error = Error;

    fn try_from(stats_data: Vec<ActionRow>) -> Result<Self, Self::Error> {
        let mut create_stats = Stats::builder();
        for row in stats_data {
            let component = &row.components[0];
            let (name, value) = match component {
                ActionRowComponent::InputText(pair) => (pair.custom_id.clone(), pair.value.clone()),
                _ => panic!("Field is not an input text field"),
            };
            let value = value
                .expect("yolo")
                .parse::<i16>()
                .map_err(|_e| Error::Plain("Failed to parse string as i16"))?;

            create_stats.set(
                name.try_into().expect("Id of input to be an attribute"),
                value,
            );
        }
        create_stats.build()
    }
}

#[derive(Default)]
pub struct StatsBuilder {
    charisma: Option<i16>,
    strength: Option<i16>,
    wisdom: Option<i16>,
    agility: Option<i16>,
}

impl StatsBuilder {
    pub fn set(&mut self, key: Attribute, value: i16) -> &mut StatsBuilder {
        match key {
            Attribute::Charisma => self.charisma = Some(value),
            Attribute::Strength => self.strength = Some(value),
            Attribute::Wisdom => self.wisdom = Some(value),
            Attribute::Agility => self.agility = Some(value),
        };
        self
    }

    pub fn build(self) -> Result<Stats, Error> {
        Ok(Stats {
            charisma: self.charisma.ok_or_else(|| Error::Plain("Oh no"))?,
            strength: self.strength.ok_or_else(|| Error::Plain("Oh no"))?,
            wisdom: self.wisdom.ok_or_else(|| Error::Plain("Oh no"))?,
            agility: self.agility.ok_or_else(|| Error::Plain("Oh no"))?,
        })
    }
}
