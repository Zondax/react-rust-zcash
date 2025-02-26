use std::ffi::{c_char, CStr};

use zcash_primitives::{
    legacy::Script,
    transaction::components::{Amount, OutPoint},
};

use crate::{
    parser::{parse_outpoint, parse_public_key, parse_script},
    ZcashError,
};

// C-compatible structs for transparent operations
#[repr(C)]
pub struct TransparentInputInfo {
    outp: *const c_char,
    pk: *const c_char,
    address: *const c_char,
    value: u64,
}

impl TransparentInputInfo {
    pub fn from_raw(
        outp: *const c_char,
        pk: *const c_char,
        address: *const c_char,
        value: u64,
    ) -> Self {
        Self {
            outp,
            pk,
            address,
            value,
        }
    }

    pub fn outpoint(&self) -> Result<OutPoint, ZcashError> {
        unsafe {
            let outp_str = CStr::from_ptr(self.outp)
                .to_str()
                .map_err(|_| ZcashError::InvalidArgument)?;

            parse_outpoint(outp_str)
        }
    }

    pub fn public_key(&self) -> Result<secp256k1::PublicKey, ZcashError> {
        unsafe {
            let pk_str = CStr::from_ptr(self.pk)
                .to_str()
                .map_err(|_| ZcashError::InvalidArgument)?;

            parse_public_key(pk_str)
        }
    }

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
        self.outp.is_null() || self.pk.is_null() || self.address.is_null()
    }
}

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JString},
    signature::{Primitive, ReturnType},
    JNIEnv,
};
#[cfg(target_os = "android")]
impl TransparentInputInfo {
    pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
        use log::error;
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

        // Create a new TransparentInputInfo with the extracted values
        let ret = Ok(TransparentInputInfo {
            outp: env.get_string(&outp_jstring).unwrap().as_ptr(),
            pk: env.get_string(&pk_jstring).unwrap().as_ptr(),
            address: env.get_string(&address_jstring).unwrap().as_ptr(),
            value: value as u64,
        });

        ret
    }
}
