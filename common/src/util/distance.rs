use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use either::Either;
use regex::Regex;

/// Distance in meters
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "SerializedDistance")]
pub struct Distance(pub f32);

/// Serialized representation of a Distance
/// Either 10.42 (float) or "10.42m" (String)
#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
struct SerializedDistance {
    #[serde(with = "either::serde_untagged")]
    meters: Either<f32, String>
}

impl Display for SerializedDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}m", self.meters)
    }
}


#[derive(thiserror::Error, Debug)]
pub struct DistanceError;

impl Display for DistanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrong distance format. Example of valid format: 42.1m")
    }
}

impl TryFrom<SerializedDistance> for Distance {
    type Error = DistanceError;

    fn try_from(value: SerializedDistance) -> Result<Self, Self::Error> {
        let meters = match value.meters {
            Either::Right(value) => {
                let regex = Regex::new(r"^(\d+\.?\d*)m?$").unwrap();
                let cleaned_value = regex.captures(&value)
                    .map(|caps| Ok(caps[1].to_string()))
                    .unwrap_or(Err(DistanceError))?;
                
                f32::from_str(cleaned_value.as_str())
                    .map_err(|_| DistanceError)
            }
            Either::Left(value) => Ok(value)
        }?;

        Ok(Self(meters))
    }
}


pub type Radius = Distance;