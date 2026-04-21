use serde::Deserialize;
use suukon::{NumeralSystem as SuukonNumeralSystem, Setting as SuukonSetting};

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct WorkspacesConfig {
    pub numeral_system: NumeralSystem,
    pub numeral_variant: NumeralVariant,
}

#[derive(Debug, Copy, Default, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NumeralSystem {
    #[default]
    Arabic,
    Japanese,
    Chinese,
    Roman,
    Egyptian,
    Rods,
}

impl From<NumeralSystem> for SuukonNumeralSystem {
    fn from(value: NumeralSystem) -> Self {
        match value {
            NumeralSystem::Arabic => SuukonNumeralSystem::Arabic,
            NumeralSystem::Japanese => SuukonNumeralSystem::Japanese,
            NumeralSystem::Chinese => SuukonNumeralSystem::Chinese,
            NumeralSystem::Roman => SuukonNumeralSystem::Roman,
            NumeralSystem::Egyptian => SuukonNumeralSystem::Egyptian,
            NumeralSystem::Rods => SuukonNumeralSystem::Rods,
        }
    }
}

#[derive(Debug, Copy, Default, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NumeralVariant {
    #[default]
    Traditional,
    Financial,
    Alt,
    DigitOnly,
    ShortNotation,
    Alpha,
    OptionalFillZero,
    Hieratic,
    HorizontalRods,
}

impl From<NumeralVariant> for SuukonSetting {
    fn from(value: NumeralVariant) -> Self {
        match value {
            NumeralVariant::Traditional => SuukonSetting::Traditional,
            NumeralVariant::Financial => SuukonSetting::Financial,
            NumeralVariant::Alt => SuukonSetting::Alt,
            NumeralVariant::DigitOnly => SuukonSetting::DigitOnly,
            NumeralVariant::ShortNotation => SuukonSetting::ShortNotation,
            NumeralVariant::Alpha => SuukonSetting::Alpha,
            NumeralVariant::OptionalFillZero => SuukonSetting::OptionalFillZero,
            NumeralVariant::Hieratic => SuukonSetting::Hieratic,
            NumeralVariant::HorizontalRods => SuukonSetting::HorizontalRods,
        }
    }
}
