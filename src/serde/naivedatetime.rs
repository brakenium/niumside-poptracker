use chrono::NaiveDateTime;
use serde::{self, Deserialize};

const ISO8601T0_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f+00:00";

pub fn serialize<S>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = format!("{}", value.format(ISO8601T0_FORMAT));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, ISO8601T0_FORMAT)
        .map_err(serde::de::Error::custom)
}