use jni::{InitArgsBuilder, JavaVM};
use once_cell::sync::Lazy;
use std::{fs::File, io::BufReader, path::Path, sync::Mutex};

pub use types::*;

use crate::{CTinData, CToutData};
use once_cell::sync::OnceCell;

mod types;

pub fn global_lock() -> &'static Mutex<()> {
    static INSTANCE: OnceCell<Mutex<()>> = OnceCell::new();
    INSTANCE.get_or_init(|| Mutex::new(()))
}

static JVM: Lazy<Mutex<Option<JavaVM>>> = Lazy::new(|| Mutex::new(None));

pub fn get_or_init_jvm() -> &'static JavaVM {
    let mut jvm_guard = JVM.lock().unwrap();

    if jvm_guard.is_none() {
        let jvm_args = InitArgsBuilder::new()
            .option("-Xcheck:jni")
            .build()
            .expect("Failed to build JVM args");

        *jvm_guard = Some(JavaVM::new(jvm_args).expect("Failed to create JavaVM"));
    }

    // This unwrap is safe because we've just ensured the Option is Some
    // The 'static lifetime is valid because JVM is a static variable
    unsafe { &*(jvm_guard.as_ref().unwrap() as *const _) }
}

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

pub fn open_test_data(name: &str) -> TestData {
    let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let file = File::open(file_path).expect("Failed to open test.json file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Failed to deserialize TestData from JSON")
}

/// Returns the salpling spend and output params
pub fn open_sapling_params() -> (String, String) {
    let spend_path_string = format!("{}/tests/sapling-spend.params", env!("CARGO_MANIFEST_DIR"));
    let output_path_string = format!("{}/tests/sapling-output.params", env!("CARGO_MANIFEST_DIR"));
    (spend_path_string, output_path_string)
}
