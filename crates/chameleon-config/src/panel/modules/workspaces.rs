use serde::Deserialize;
use suukon::{NumeralSystem as SuukonNumeralSystem, Setting as SuukonSetting};

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct WorkspacesConfig {
    #[serde(default)]
    pub numeral_system: NumeralSystem,

    #[serde(default)]
    pub numeral_variant: NumeralVariant,
}

#[derive(Debug, Copy, Default, Deserialize, PartialEq, Clone)]
pub enum NumeralSystem {
    #[default]
    #[serde(rename = "arabic")]
    Arabic,
    #[serde(rename = "japanese")]
    Japanese,
    #[serde(rename = "chinese")]
    Chinese,
    #[serde(rename = "roman")]
    Roman,
    #[serde(rename = "egyptian")]
    Egyptian,
    #[serde(rename = "rods")]
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
pub enum NumeralVariant {
    #[default]
    #[serde(rename = "traditional")]
    Traditional,
    #[serde(rename = "financial")]
    Financial,
    #[serde(rename = "alt")]
    Alt,
    #[serde(rename = "digit_only")]
    DigitOnly,
    #[serde(rename = "short_notation")]
    ShortNotation,
    #[serde(rename = "alpha")]
    Alpha,
    #[serde(rename = "optional_fill_zero")]
    OptionalFillZero,
    #[serde(rename = "hieratic")]
    Hieratic,
    #[serde(rename = "horizontal_rods")]
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
