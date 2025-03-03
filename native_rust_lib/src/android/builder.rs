use crate::{
    add_signatures, add_transparent_input, add_transparent_output, build_transaction,
    create_builder, destroy_builder,
    error::{get_error_description, ZcashError},
    free_transaction_data, TransactionSignatures, TransparentInputInfo, TransparentOutputInfo,
};

use jni::{
    objects::{JClass, JObject, JString},
    sys::{jbyteArray, jint, jlong},
    JNIEnv,
};

use log::{error, info};

use crate::init_logger;
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_createBuilder(
    mut env: JNIEnv,
    _class: JClass,
    fee: jlong,
    network_type: jint,
) -> jlong {
    init_logger();
    info!(
        "Creating builder with network type: {} and fee: {}",
        network_type, fee
    );

    // Variable to hold the builder ID
    let mut builder_id: u64 = 0;

    // Call the Rust create_builder function
    let result = create_builder(fee as u64, network_type as u8, &mut builder_id);

    if result != ZcashError::Success as u32 {
        // Handle error
        let error_msg = get_error_description(result);
        error!("Error creating builder: {}", error_msg);

        // Throw Java exception
        let exception_class = env.find_class("java/lang/RuntimeException").unwrap();
        env.throw_new(
            exception_class,
            format!("Failed to create builder: {}", error_msg),
        )
        .unwrap();

        return -1;
    }

    // Return the builder ID on success
    builder_id as jlong
}

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_destroyBuilder(
    _env: JNIEnv,
    _class: JClass,
    builder_id: jlong,
) -> jint {
    init_logger();
    info!("Destroying builder: {}", builder_id);

    let result = destroy_builder(builder_id as u64);

    if result != ZcashError::Success as u32 {
        let error_msg = get_error_description(result);
        error!("Error destroying builder: {}", error_msg);
    }

    result as jint
}

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_addTransparentInput(
    mut env: JNIEnv,
    _class: JClass,
    builder_id: jlong,
    input_obj: JObject,
) -> jint {
    init_logger();
    info!("Adding transparent input to builder: {}", builder_id);

    // Create a TransparentInputInfo from the Java object
    let input = match TransparentInputInfo::from_java(&mut env, input_obj) {
        Ok(input) => input,
        Err(e) => return e as jint,
    };

    let result = add_transparent_input(builder_id as u64, input);

    if result != ZcashError::Success as u32 {
        let error_msg = get_error_description(result);
        error!("Error destroying builder: {}", error_msg);
    }

    result as jint
}

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_addTransparentOutput(
    mut env: JNIEnv,
    _class: JClass,
    builder_id: jlong,
    output_obj: JObject,
) -> jint {
    init_logger();
    info!("Adding transparent output to builder: {}", builder_id);

    // Create a TransparentOutputInfo from the Java object
    let output = match TransparentOutputInfo::from_java(&mut env, output_obj) {
        Ok(output) => output,
        Err(e) => return e as jint,
    };

    let result = add_transparent_output(builder_id as u64, output);

    if result != ZcashError::Success as u32 {
        let error_msg = get_error_description(result);
        error!("Error destroying builder: {}", error_msg);
    }

    result as jint
}

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_addSignatures(
    mut env: JNIEnv,
    _class: JClass,
    builder_id: jlong,
    signatures_obj: JObject,
) -> jint {
    init_logger();
    info!("Adding signatures to builder: {}", builder_id);

    // Create a TransactionSignatures from the Java object
    let signatures = match TransactionSignatures::from_java(&mut env, signatures_obj) {
        Ok(signatures) => signatures,
        Err(e) => return e as jint,
    };

    let result = add_signatures(builder_id as u64, signatures);

    if result != ZcashError::Success as u32 {
        let error_msg = get_error_description(result);
        error!("Error destroying builder: {}", error_msg);
    }

    result as jint
}

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_buildTransaction(
    mut env: JNIEnv,
    _class: JClass,
    builder_id: jlong,
    spend_path: JString,
    output_path: JString,
    tx_version: jint,
) -> jbyteArray {
    use std::ffi::CStr;

    init_logger();
    info!(
        "MyRustModule: Building transaction for builder: {}",
        builder_id
    );

    // Keep the JavaStr objects alive
    let spend_path_jstr = env.get_string(&spend_path).unwrap();
    let output_path_jstr = env.get_string(&output_path).unwrap();

    // Print the extracted strings for debugging
    let spend_path_str = CStr::from_ptr(spend_path_jstr.as_ptr()).to_str().unwrap();
    let output_path_str = CStr::from_ptr(output_path_jstr.as_ptr()).to_str().unwrap();
    info!("MyRustModule: Extracted spend path: {}", spend_path_str);
    info!("MyRustModule: Extracted output path: {}", output_path_str);

    // Use the pointers while the JavaStr objects are still alive
    let spend_path_ptr = spend_path_jstr.as_ptr();
    let output_path_ptr = output_path_jstr.as_ptr();

    info!("MyRustModule:got path pointers");
    let mut result_ptr: *mut u8 = ptr::null_mut();
    let mut result_len: usize = 0;
    info!("MyRustModule:Calling rust-native build_transaction");

    let result = build_transaction(
        builder_id as u64,
        spend_path_ptr,
        output_path_ptr,
        tx_version as u8,
        &mut result_ptr,
        &mut result_len,
    );

    info!("MyRustModule:Transaction build result: {}", result);
    info!("MyRustModule:Transaction data length {}", result_len);

    if result == ZcashError::Success as u32 && !result_ptr.is_null() {
        let byte_array = env.new_byte_array(result_len as jint).unwrap();
        env.set_byte_array_region(
            &byte_array,
            0,
            std::slice::from_raw_parts(result_ptr as *const i8, result_len),
        )
        .unwrap();
        // Free the data allocated in build_transaction
        free_transaction_data(result_ptr);
        **byte_array
    } else {
        let error_msg = get_error_description(result);
        error!("Error destroying builder: {}", error_msg);
        **env.new_byte_array(0).unwrap()
    }
}
