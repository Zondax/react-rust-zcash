use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;

// Custom deserializer function for CString
pub fn deserialize_cstring<'de, D>(deserializer: D) -> Result<CString, D::Error>
where
    D: Deserializer<'de>,
{
    // Define a visitor that converts a string to CString
    struct CStringVisitor(PhantomData<fn() -> CString>);

    impl<'de> Visitor<'de> for CStringVisitor {
        type Value = CString;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            CString::new(value)
                .map_err(|_| E::custom("CString conversion error: contains null byte"))
        }
    }

    deserializer.deserialize_str(CStringVisitor(PhantomData))
}

pub fn deserialize_cstring_vec<'de, D>(deserializer: D) -> Result<Vec<CString>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize to a Vec<String> first
    let string_vec: Vec<String> = Deserialize::deserialize(deserializer)?;

    // Then convert each String to a CString
    string_vec
        .into_iter()
        .map(|s| CString::new(s).map_err(|_| de::Error::custom("CString conversion error")))
        .collect()
}
