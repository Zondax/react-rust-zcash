use jni::errors::Result as JniResult;
use jni::objects::{JObject, JValue};
use jni::JNIEnv;
mod types;
pub use types::*;
mod deserializer;
pub use deserializer::*;

use crate::{CTinData, CToutData};

impl<'a> From<&'a TInput> for CTinData {
    fn from(input: &'a TInput) -> Self {
        CTinData {
            path: input.path.as_ptr(),
            path_len: input.path.len(),
            address: input.address.as_ptr(),
            value: input.value,
        }
    }
}

impl<'a> From<&'a TOutput> for CToutData {
    fn from(output: &'a TOutput) -> Self {
        CToutData {
            address: output.address.as_ptr(),
            value: output.value,
        }
    }
}
