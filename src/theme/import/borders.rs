use ratatui::widgets::BorderType;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct BorderScheme {
    pub display: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_border_type")]
    pub style: Option<BorderType>,
}

// Allows for case-insenstive matching
fn deserialize_border_type<'de, D>(deserializer: D) -> Result<Option<BorderType>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(s) = Option::<String>::deserialize(deserializer)? else {
        return Ok(None);
    };

    // Remove common separators and compare lowercase
    let normalized: String = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();

    Ok(Some(match normalized.as_str() {
        "plain" => BorderType::Plain,
        "rounded" => BorderType::Rounded,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        "lightdoubledashed" => BorderType::LightDoubleDashed,
        "heavydoubledashed" => BorderType::HeavyDoubleDashed,
        "lighttripledashed" => BorderType::LightTripleDashed,
        "heavytripledashed" => BorderType::HeavyTripleDashed,
        "lightquadrupledashed" => BorderType::LightQuadrupleDashed,
        "heavyquadrupledashed" => BorderType::HeavyQuadrupleDashed,
        "quadrantinside" => BorderType::QuadrantInside,
        "quadrantoutside" => BorderType::QuadrantOutside,
        _ => BorderType::Rounded,
    }))
}
