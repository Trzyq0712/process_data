use serde::{de, Deserialize, Deserializer};
use std::fmt;

use crate::point::Point;

pub type RssArr = Vec<f32>;

#[derive(Debug, Clone)]
pub struct RssRecord {
    pub point: Point,
    pub rss: RssArr,
}

impl<'de> Deserialize<'de> for RssRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RssRecordVisitor)
    }
}

struct RssRecordVisitor;

impl<'de> de::Visitor<'de> for RssRecordVisitor {
    type Value = RssRecord;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct RssRecord")
    }

    fn visit_map<V>(self, mut map: V) -> Result<RssRecord, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut x = None;
        let mut y = None;
        let mut leds = Vec::new();

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "x" => {
                    x = Some(map.next_value::<f32>()? as u32);
                }
                "y" => {
                    y = Some(map.next_value::<f32>()? as u32);
                }
                k if k.starts_with("led_") => {
                    leds.push(map.next_value::<f32>()?);
                }
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
        let y = y.ok_or_else(|| de::Error::missing_field("y"))?;

        Ok(RssRecord {
            point: Point { x, y },
            rss: leds,
        })
    }
}
