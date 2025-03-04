use crate::deserialize_cstring;
use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::{ffi::CTransparentInput, ZcashError};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransparentInput {
    #[serde(deserialize_with = "deserialize_cstring")]
    pub outp: CString,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub pk: CString,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub address: CString,
    pub value: u64,
}

impl TransparentInput {
    pub fn as_raw(&self) -> CTransparentInput {
        CTransparentInput::from_raw(
            self.outp.as_ptr(),
            self.pk.as_ptr(),
            self.address.as_ptr(),
            self.value,
        )
    }
}

#[cfg(any(target_os = "android", test))]
use jni::{
    objects::{JObject, JString},
    signature::{Primitive, ReturnType},
    JNIEnv,
};

#[cfg(any(target_os = "android", test))]
impl TransparentInput {
    pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
        use log::error;
        use std::ffi::CString;

        // Get outp field
        let outp_field = match env.get_field_id(
            env.get_object_class(&obj).unwrap(),
            "outp",
            "Ljava/lang/String;",
        ) {
            Ok(id) => id,
            Err(_) => {
                error!("Error getting outp field");
                return Err(ZcashError::InvalidArgument);
            }
        };

        // Use ReturnType::Object for string fields
        let outp_value = match env.get_field_unchecked(&obj, outp_field, ReturnType::Object) {
            Ok(value) => value.l().unwrap(),
            Err(_) => {
                error!("Error getting outp field");
                return Err(ZcashError::InvalidArgument);
            }
        };
        let outp_jstring = JString::from(outp_value);
        if outp_jstring.is_null() {
            return Err(ZcashError::InvalidArgument);
        }

        // Extract string data safely
        let outp_rust_string = env
            .get_string(&outp_jstring)
            .map_err(|_| ZcashError::InvalidArgument)?
            .to_string_lossy()
            .into_owned();

        // Get pk field
        let pk_field = match env.get_field_id(
            env.get_object_class(&obj).unwrap(),
            "pk",
            "Ljava/lang/String;",
        ) {
            Ok(id) => id,
            Err(_) => return Err(ZcashError::InvalidArgument),
        };

        // Use ReturnType::Object for string fields
        let pk_value = match env.get_field_unchecked(&obj, pk_field, ReturnType::Object) {
            Ok(value) => value.l().unwrap(),
            Err(_) => return Err(ZcashError::InvalidArgument),
        };
        let pk_jstring = JString::from(pk_value);
        if pk_jstring.is_null() {
            return Err(ZcashError::InvalidArgument);
        }

        // Extract string data safely
        let pk_rust_string = env
            .get_string(&pk_jstring)
            .map_err(|_| ZcashError::InvalidArgument)?
            .to_string_lossy()
            .into_owned();

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
            Err(_) => return Err(ZcashError::InvalidArgument),
        };
        let address_jstring = JString::from(address_value);
        if address_jstring.is_null() {
            return Err(ZcashError::InvalidArgument);
        }

        // Extract string data safely
        let address_rust_string = env
            .get_string(&address_jstring)
            .map_err(|_| ZcashError::InvalidArgument)?
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
            .map_err(|_| ZcashError::InvalidArgument)?;

        // Create CStrings from the Rust strings
        let outp_cstring =
            CString::new(outp_rust_string).map_err(|_| ZcashError::InvalidOutpoint)?;
        let pk_cstring = CString::new(pk_rust_string).map_err(|_| ZcashError::InvalidPubkey)?;
        let address_cstring =
            CString::new(address_rust_string).map_err(|_| ZcashError::InvalidAddress)?;

        // Create a new TransparentInputInfo with the extracted values
        Ok(TransparentInput {
            outp: outp_cstring,
            pk: pk_cstring,
            address: address_cstring,
            value,
        })
    }
}
