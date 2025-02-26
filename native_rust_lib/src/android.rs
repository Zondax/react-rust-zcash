use crate::error::get_error_description;

use jni::{
    objects::JClass,
    sys::{jint, jstring},
    JNIEnv,
};

mod builder;
pub use builder::*;
mod fee;
pub use fee::*;
mod init_data;
pub use init_data::*;

pub(crate) fn init_logger() {
    use android_logger::Config;
    use log::LevelFilter;

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("RustModule"),
    );
}

#[cfg(target_os = "android")]
pub fn create_java_string(env: &JNIEnv, desc_str: &str) -> jstring {
    // Fix: Use into_raw() instead of into_inner()
    env.new_string(desc_str).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_getErrorDescription(
    env: JNIEnv,
    _class: JClass,
    error_code: jint,
) -> jstring {
    init_logger();

    let desc_str = get_error_description(error_code as u32);
    create_java_string(&env, &desc_str)
}
