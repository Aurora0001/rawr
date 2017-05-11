use std::fmt;

use serde::de::{self, Visitor, Deserializer};

#[doc(ignore)]
/// For some  reason, Reddit sometimes sends its timestamps as floats and sometimes as integers. So
/// we need some custom logic to parse them, because serde complains otherwise.
pub fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where D: Deserializer<'de>
{
    struct TsVisitor;

    impl<'de> Visitor<'de> for TsVisitor {
        type Value = i64;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "A timestamp that's either a float or an integer")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(v)
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(v as i64)
        }
    }

    deserializer.deserialize_any(TsVisitor)
}
