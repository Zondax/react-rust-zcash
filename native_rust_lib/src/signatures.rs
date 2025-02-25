use secp256k1::ecdsa::Signature;
use std::ffi::{c_char, CStr};

use crate::ZcashError;

// Structure for collecting signatures for a transaction
#[repr(C)]
pub struct TransactionSignatures {
    pub transparent_sigs: *const *const c_char,
    pub transparent_sigs_len: usize,
}

impl TransactionSignatures {
    pub fn transparent_sigs(&self) -> Result<Vec<Signature>, ZcashError> {
        let transparent_signatures =
            if self.transparent_sigs_len > 0 && !self.transparent_sigs.is_null() {
                let sigs_ptr = unsafe {
                    std::slice::from_raw_parts(self.transparent_sigs, self.transparent_sigs_len)
                };

                let mut parsed_sigs = Vec::with_capacity(sigs_ptr.len());
                for sig_ptr in sigs_ptr {
                    if sig_ptr.is_null() {
                        return Err(ZcashError::InvalidArgument);
                    }

                    let sig_str = unsafe {
                        CStr::from_ptr(*sig_ptr)
                            .to_str()
                            .map_err(|_| ZcashError::InvalidArgument)?
                    };

                    // Parse DER signature
                    let bytes = hex::decode(sig_str).map_err(|_| ZcashError::InvalidArgument)?;

                    let Ok(sig) = secp256k1::ecdsa::Signature::from_der(&bytes) else {
                        return Err(ZcashError::InvalidArgument);
                    };
                    parsed_sigs.push(sig);
                }
                parsed_sigs
            } else {
                vec![]
            };
        Ok(transparent_signatures)
    }
}

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JObjectArray, JString},
    signature::ReturnType,
    JNIEnv,
};
#[cfg(target_os = "android")]
impl TransactionSignatures {
    pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
        use std::ffi::CString;

        // Get transparent_sigs field (array of strings)
        let transparent_sigs_field = match env.get_field_id(
            env.get_object_class(&obj).unwrap(),
            "transparentSigs",
            "[Ljava/lang/String;",
        ) {
            Ok(id) => id,
            Err(_) => return Err(ZcashError::InvalidArgument),
        };

        // Use ReturnType::Array for array fields
        let transparent_sigs_array =
            match env.get_field_unchecked(obj, transparent_sigs_field, ReturnType::Array) {
                Ok(value) => value.l().unwrap(),
                Err(_) => return Err(ZcashError::InvalidArgument),
            };

        if transparent_sigs_array.is_null() {
            // If array is null, return empty signatures
            return Ok(TransactionSignatures {
                transparent_sigs: std::ptr::null(),
                transparent_sigs_len: 0,
            });
        }

        // Convert JObject to JObjectArray
        let object_array = JObjectArray::from(transparent_sigs_array);

        // Get array length - use the JObjectArray reference
        let array_length = env.get_array_length(&object_array).unwrap_or(0) as usize;

        if array_length == 0 {
            // If array is empty, return empty signatures
            return Ok(TransactionSignatures {
                transparent_sigs: std::ptr::null(),
                transparent_sigs_len: 0,
            });
        }

        // Allocate memory for string pointers
        let mut c_strings = Vec::with_capacity(array_length);
        let mut string_ptrs = Vec::with_capacity(array_length);

        // Extract strings from Java array
        for i in 0..array_length {
            // Use object_array rather than transparent_sigs_array
            let string_obj = env
                .get_object_array_element(&object_array, i as i32)
                .unwrap();

            if string_obj.is_null() {
                continue;
            }

            let jstring = JString::from(string_obj);
            let rust_string = match env.get_string(&jstring) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Convert to C string and store in vector to prevent it from being dropped
            let c_string = match CString::new(rust_string.to_str().unwrap()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            string_ptrs.push(c_string.as_ptr());
            c_strings.push(c_string);
        }

        // If no valid strings were found, return empty signatures
        if string_ptrs.is_empty() {
            return Ok(TransactionSignatures {
                transparent_sigs: std::ptr::null(),
                transparent_sigs_len: 0,
            });
        }

        // Create a new TransactionSignatures with the extracted values
        Ok(TransactionSignatures {
            transparent_sigs: string_ptrs.as_ptr(),
            transparent_sigs_len: string_ptrs.len(),
        })
    }
}
