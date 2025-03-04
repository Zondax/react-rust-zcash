use crate::{create_java_string, error::get_error_description, init_logger};

use jni::{
    objects::JClass,
    sys::{jint, jstring},
    JNIEnv,
};

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
