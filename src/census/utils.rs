use crate::census::constants::{CharacterID, WorldID};
use chrono::Duration;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;

#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::ref_option)]
pub fn serialize_optional_bool<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(value) => match value {
            true => serializer.serialize_str("true"),
            false => serializer.serialize_str("false"),
        },
    }
}

pub fn serialize_all_subscription<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    json!(["all"]).serialize(serializer)
}

pub fn serialize_char_ids_subscription<S>(
    value: &[CharacterID],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut ids = Vec::with_capacity(value.len());
    for id in value {
        ids.push(id.to_string());
    }

    serializer.collect_seq(ids.iter())
}

pub fn serialize_world_ids_subscription<S>(
    value: &Vec<WorldID>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut ids = Vec::with_capacity(value.len());
    for id in value {
        ids.push((*id as u8).to_string());
    }

    serializer.collect_seq(ids.iter())
}

pub fn deserialize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

pub fn deserialize_from_str_custom_impl<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

pub fn deserialize_duration_from_str<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let duration = String::deserialize(deserializer)?
        .parse::<i64>()
        .map_err(serde::de::Error::custom)?;

    Ok(Duration::seconds(duration))
}

pub fn serialize_duration<S>(duration: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i64(duration.num_seconds())
}

pub fn de_bool_from_str_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let int_value = string.parse::<u8>().map_err(|_| {
        serde::de::Error::invalid_type(serde::de::Unexpected::Str(&string), &"Not an int")
    })?;

    match int_value {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(u64::from(other)),
            &"zero or one",
        )),
    }
}
