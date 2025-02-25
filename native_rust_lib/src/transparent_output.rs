use std::ffi::{c_char, CStr};

use zcash_primitives::{legacy::Script, transaction::components::Amount};

use crate::{parser::parse_script, ZcashError};

#[repr(C)]
pub struct TransparentOutputInfo {
    address: *const c_char,
    value: u64,
}

impl TransparentOutputInfo {
    pub fn address(&self) -> Result<Script, ZcashError> {
        unsafe {
            let address_str = CStr::from_ptr(self.address)
                .to_str()
                .map_err(|_| ZcashError::InvalidArgument)?;
            parse_script(address_str)
        }
    }

    pub fn amount(&self) -> Result<Amount, ZcashError> {
        Amount::from_u64(self.value).map_err(|_| ZcashError::InvalidArgument)
    }

    pub fn any_null(&self) -> bool {
        self.address.is_null()
    }
}

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JString},
    signature::{Primitive, ReturnType},
    JNIEnv,
};
#[cfg(target_os = "android")]
impl TransparentOutputInfo {
    pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
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

        // Create a new TransparentOutputInfo with the extracted values
        let ret = Ok(TransparentOutputInfo {
            address: env.get_string(&address_jstring).unwrap().as_ptr(),
            value: value as u64,
        });
        ret
    }
}
