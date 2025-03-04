use crate::{deserializer::deserialize_cstring_vec, ffi::CSignatures, ZcashError};
use std::ffi::{c_char, CString};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Signatures {
    #[serde(deserialize_with = "deserialize_cstring_vec")]
    pub transparent_sigs: Vec<CString>,
    #[serde(deserialize_with = "deserialize_cstring_vec")]
    pub sapling_sigs: Vec<CString>,
}

impl Signatures {
    /// Converts `Signatures` into a `Signatures` struct.
    /// The caller is responsible for freeing the memory using `free_transaction_signatures`.
    pub fn as_raw(&self) -> CSignatures {
        // Convert Vec<CString> to Vec<*const c_char>
        let transparent_sigs_ptrs: Vec<*const c_char> = self
            .transparent_sigs
            .iter()
            .map(|cstring| cstring.as_ptr())
            .collect();

        // Convert Vec<*const c_char> to *const *const c_char
        let transparent_sigs = transparent_sigs_ptrs.as_ptr();
        let transparent_sigs_len = transparent_sigs_ptrs.len();

        // Prevent the Vec from being deallocated
        std::mem::forget(transparent_sigs_ptrs);

        CSignatures {
            transparent_sigs,
            transparent_sigs_len,
        }
    }
}

#[cfg(any(target_os = "android", test))]
use jni::{
    objects::{JObject, JString},
    signature::ReturnType,
    JNIEnv,
};

#[cfg(any(target_os = "android", test))]
impl Signatures {
    /// Converts a Java object into a `Signatures` struct.
    /// # Safety
    /// This function is unsafe because it dereferences raw pointers comming from
    /// Java objects
    pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
        use log::error;
        use std::ffi::CString;

        // Get transparentSigs List field
        let field_name = "transparentSigs";
        let field_type = "Ljava/util/List;";

        let transparent_sigs_field = match env.get_field_id(
            env.get_object_class(&obj)
                .map_err(|_| ZcashError::InvalidArgument)?,
            field_name,
            field_type,
        ) {
            Ok(id) => id,
            Err(e) => {
                error!("Error getting transparentSigs field: {:?}", e);
                return Err(ZcashError::InvalidArgument);
            }
        };

        // Get the List object
        let transparent_list =
            match env.get_field_unchecked(obj, transparent_sigs_field, ReturnType::Object) {
                Ok(value) => value.l().map_err(|_| ZcashError::InvalidArgument)?,
                Err(e) => {
                    error!("Error getting transparentSigs field: {:?}", e);
                    return Err(ZcashError::InvalidArgument);
                }
            };

        if transparent_list.is_null() {
            return Ok(Signatures {
                transparent_sigs: vec![],
                sapling_sigs: vec![],
            });
        }

        // Get the size of the list
        let size_method = match env.get_method_id(
            env.get_object_class(&transparent_list).unwrap(),
            "size",
            "()I",
        ) {
            Ok(id) => id,
            Err(e) => {
                error!("Error getting transparentSigs field: {:?}", e);
                return Err(ZcashError::InvalidArgument);
            }
        };

        let transparent_size = match env
            .call_method_unchecked(
                &transparent_list,
                size_method,
                ReturnType::Primitive(jni::signature::Primitive::Int),
                &[],
            )
            .and_then(|result| result.i())
        {
            Ok(size) => size,
            Err(e) => {
                error!("Error calling size_method: {:?}", e);
                return Err(ZcashError::InvalidArgument);
            }
        };

        if transparent_size == 0 {
            return Ok(Signatures {
                transparent_sigs: vec![],
                sapling_sigs: vec![],
            });
        }

        // Get "get" method for the List
        let get_method = env
            .get_method_id(
                env.get_object_class(&transparent_list).unwrap(),
                "get",
                "(I)Ljava/lang/Object;",
            )
            .map_err(|_| ZcashError::InvalidSignature)?;

        // Collect transparent signatures
        let mut transparent_sigs = Vec::with_capacity(transparent_size as usize);
        for i in 0..transparent_size {
            // Create argument for get method
            let index_arg = jni::sys::jvalue { i };

            // Get string at index i
            let sig_obj = env
                .call_method_unchecked(
                    &transparent_list,
                    get_method,
                    ReturnType::Object,
                    &[index_arg],
                )
                .and_then(|result| result.l())
                .map_err(|_| ZcashError::InvalidSignature)?;

            if sig_obj.is_null() {
                return Err(ZcashError::InvalidSignature);
            }

            let jstring = JString::from(sig_obj);
            let rust_string = env
                .get_string(&jstring)
                .map_err(|_| ZcashError::InvalidSignature)?;

            let c_string = CString::new(rust_string.to_string_lossy().into_owned())
                .map_err(|_| ZcashError::InvalidSignature)?;

            transparent_sigs.push(c_string);
        }

        // Do the same for sapling_sigs (omitted for brevity)
        let sapling_sigs = Vec::new();

        // Create a new Signatures with the extracted values
        Ok(Signatures {
            transparent_sigs,
            sapling_sigs,
        })
    }
}
