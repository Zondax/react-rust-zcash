use crate::{deserialize_cstring, ZcashError};
use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::ffi::CTransparentOutput;

#[derive(Debug, Serialize, Deserialize)]
pub struct TransparentOutput {
    #[serde(deserialize_with = "deserialize_cstring")]
    pub address: CString,
    pub value: u64,
}

impl TransparentOutput {
    pub fn as_raw(&self) -> CTransparentOutput {
        CTransparentOutput::from_raw(self.address.as_ptr(), self.value)
    }
}

#[cfg(any(target_os = "android", test))]
use jni::{
    objects::{JObject, JString},
    signature::{Primitive, ReturnType},
    JNIEnv,
};
#[cfg(any(target_os = "android", test))]
impl TransparentOutput {
    pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
        use std::ffi::CString;

        // Get address field
        let address_field = match env.get_field_id(
            env.get_object_class(&obj).unwrap(),
            "address",
            "Ljava/lang/String;",
        ) {
            Ok(id) => id,
            Err(_) => return Err(ZcashError::InvalidArgument),
        };

        // Use ReturnType::Object for string fields
        let address_value = match env.get_field_unchecked(&obj, address_field, ReturnType::Object) {
            Ok(value) => value.l().unwrap(),
            Err(_) => return Err(ZcashError::InvalidAddress),
        };

        let address_jstring = JString::from(address_value);
        if address_jstring.is_null() {
            return Err(ZcashError::InvalidAddress);
        }

        // Extract string data safely into a Rust string
        let address_rust_string = env
            .get_string(&address_jstring)
            .map_err(|_| ZcashError::InvalidAddress)?
            .to_string_lossy()
            .into_owned();

        // Get value field
        let value_field = match env.get_field_id(env.get_object_class(&obj).unwrap(), "value", "J")
        {
            Ok(id) => id,
            Err(_) => return Err(ZcashError::InvalidArgument),
        };

        // Use ReturnType::Primitive for primitive types
        let value = env
            .get_field_unchecked(&obj, value_field, ReturnType::Primitive(Primitive::Long))
            .and_then(|value| value.j())
            .map(|long_value| long_value as u64)
            .map_err(|_| ZcashError::InvalidTxValue)?;

        // Create CString from the Rust string
        let address_cstring =
            CString::new(address_rust_string).map_err(|_| ZcashError::InvalidAddress)?;

        // Create a new TransparentOutputInfo with the extracted values
        Ok(TransparentOutput {
            address: address_cstring,
            value,
        })
    }
}
