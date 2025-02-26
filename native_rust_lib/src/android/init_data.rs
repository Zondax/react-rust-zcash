use crate::android::init_logger;
use crate::memory::free_transaction_data;
use crate::ZcashError;
use jni::{
    objects::{JByteArray, JClass, JIntArray, JObject, JString, JValue},
    sys::{jbyteArray, jint, jsize},
    JNIEnv,
};

use log::{error, info};
use std::ffi::CString;

use crate::init_data::{
    get_inittx_data, CInitData, CSaplingInData, CSaplingOutData, CTinData, CToutData,
};

// We can not move each conversion section to its own Type::from_java
// due to lifetimes issues, so we parse the java object directly here and compute
// the init_data as well
#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_getInitTxData(
    mut env: JNIEnv,
    _class: JClass,
    init_obj: JObject,
) -> jbyteArray {
    init_logger();
    info!("Processing InitData from Java object");

    // Get arrays from the Java object
    let t_in_list = match env.get_field(&init_obj, "tIn", "Ljava/util/List;") {
        Ok(field) => match field.l() {
            Ok(obj) => obj,
            Err(_) => {
                error!("Failed to get tIn field as object");
                return std::ptr::null_mut();
            }
        },
        Err(_) => {
            error!("Failed to get tIn field");
            return std::ptr::null_mut();
        }
    };

    let t_out_list = match env.get_field(&init_obj, "tOut", "Ljava/util/List;") {
        Ok(field) => match field.l() {
            Ok(obj) => obj,
            Err(_) => {
                error!("Failed to get tOut field as object");
                return std::ptr::null_mut();
            }
        },
        Err(_) => {
            error!("Failed to get tOut field");
            return std::ptr::null_mut();
        }
    };

    let s_spend_list = match env.get_field(&init_obj, "sSpend", "Ljava/util/List;") {
        Ok(field) => match field.l() {
            Ok(obj) => obj,
            Err(_) => {
                error!("Failed to get sSpend field as object");
                return std::ptr::null_mut();
            }
        },
        Err(_) => {
            error!("Failed to get sSpend field");
            return std::ptr::null_mut();
        }
    };

    let s_output_list = match env.get_field(&init_obj, "sOutput", "Ljava/util/List;") {
        Ok(field) => match field.l() {
            Ok(obj) => obj,
            Err(_) => {
                error!("Failed to get sOutput field as object");
                return std::ptr::null_mut();
            }
        },
        Err(_) => {
            error!("Failed to get sOutput field");
            return std::ptr::null_mut();
        }
    };

    // Get list sizes
    let t_in_size = match env.call_method(&t_in_list, "size", "()I", &[]) {
        Ok(val) => val.i().unwrap_or(0) as usize,
        Err(_) => 0,
    };

    let t_out_size = match env.call_method(&t_out_list, "size", "()I", &[]) {
        Ok(val) => val.i().unwrap_or(0) as usize,
        Err(_) => 0,
    };

    let s_spend_size = match env.call_method(&s_spend_list, "size", "()I", &[]) {
        Ok(val) => val.i().unwrap_or(0) as usize,
        Err(_) => 0,
    };

    let s_output_size = match env.call_method(&s_output_list, "size", "()I", &[]) {
        Ok(val) => val.i().unwrap_or(0) as usize,
        Err(_) => 0,
    };

    // Create vectors to store the C structs
    let mut t_in_vec = Vec::with_capacity(t_in_size);
    let mut t_out_vec = Vec::with_capacity(t_out_size);
    let mut s_spend_vec = Vec::with_capacity(s_spend_size);
    let mut s_output_vec = Vec::with_capacity(s_output_size);

    // Storage for data that needs to stay alive (for lifetime management)
    let mut path_storage: Vec<Vec<u32>> = Vec::with_capacity(t_in_size);
    let mut address_storage: Vec<CString> =
        Vec::with_capacity(t_in_size + t_out_size + s_spend_size + s_output_size);
    let mut ovk_storage: Vec<Vec<u8>> = Vec::with_capacity(s_output_size);

    // Process transparent inputs
    for i in 0..t_in_size {
        let item = match env.call_method(
            &t_in_list,
            "get",
            "(I)Ljava/lang/Object;",
            &[JValue::Int(i as jint)],
        ) {
            Ok(val) => match val.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if item.is_null() {
            continue;
        }

        // Get the 'path' field as a Java int array
        let path_obj = match env.get_field(&item, "path", "[I") {
            Ok(field) => match field.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if path_obj.is_null() {
            continue;
        }

        // Cast to JIntArray so that it implements AsJArrayRaw
        let path_array = JIntArray::from(path_obj);

        let path_len = match env.get_array_length(&path_array) {
            Ok(len) => len as usize,
            Err(_) => continue,
        };

        if path_len != 5 {
            continue;
        }

        let mut path_elements = vec![0i32; 5];
        if let Err(_) = env.get_int_array_region(&path_array, 0, &mut path_elements) {
            continue;
        }

        let path_u32: Vec<u32> = path_elements.iter().map(|&x| x as u32).collect();

        // Get address string
        let address_field = match env.get_field(&item, "address", "Ljava/lang/String;") {
            Ok(field) => match field.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if address_field.is_null() {
            continue;
        }

        let j_string = JString::from(address_field);
        let r_string = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_str = match r_string.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_cstr = match CString::new(address_str) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Get value
        let value = match env.get_field(&item, "value", "J") {
            Ok(field) => field.j().unwrap_or(0) as u64,
            Err(_) => continue,
        };

        // Keep the path vector and address alive
        path_storage.push(path_u32);
        address_storage.push(address_cstr);

        // Create and add the CTinData struct
        t_in_vec.push(CTinData {
            path: path_storage.last().unwrap().as_ptr(),
            path_len: 5,
            address: address_storage.last().unwrap().as_ptr(),
            value,
        });
    }

    // Process transparent outputs
    for i in 0..t_out_size {
        let item = match env.call_method(
            &t_out_list,
            "get",
            "(I)Ljava/lang/Object;",
            &[JValue::Int(i as jint)],
        ) {
            Ok(val) => match val.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if item.is_null() {
            continue;
        }

        // Get address string
        let address_field = match env.get_field(&item, "address", "Ljava/lang/String;") {
            Ok(field) => match field.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if address_field.is_null() {
            continue;
        }

        let j_string = JString::from(address_field);
        let r_string = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_str = match r_string.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_cstr = match CString::new(address_str) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Get value
        let value = match env.get_field(&item, "value", "J") {
            Ok(field) => field.j().unwrap_or(0) as u64,
            Err(_) => continue,
        };

        // Store and add the CToutData struct
        address_storage.push(address_cstr);

        t_out_vec.push(CToutData {
            address: address_storage.last().unwrap().as_ptr(),
            value,
        });
    }

    // Process sapling spends
    for i in 0..s_spend_size {
        let item = match env.call_method(
            &s_spend_list,
            "get",
            "(I)Ljava/lang/Object;",
            &[JValue::Int(i as jint)],
        ) {
            Ok(val) => match val.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if item.is_null() {
            continue;
        }

        // Get 'path' as a single int value
        let path = match env.get_field(&item, "path", "I") {
            Ok(field) => field.i().unwrap_or(0) as u32,
            Err(_) => continue,
        };

        // Get address string
        let address_field = match env.get_field(&item, "address", "Ljava/lang/String;") {
            Ok(field) => match field.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if address_field.is_null() {
            continue;
        }

        let j_string = JString::from(address_field);
        let r_string = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_str = match r_string.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_cstr = match CString::new(address_str) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Get value
        let value = match env.get_field(&item, "value", "J") {
            Ok(field) => field.j().unwrap_or(0) as u64,
            Err(_) => continue,
        };

        address_storage.push(address_cstr);

        s_spend_vec.push(CSaplingInData {
            path,
            address: address_storage.last().unwrap().as_ptr(),
            value,
        });
    }

    // Process sapling outputs
    for i in 0..s_output_size {
        let item = match env.call_method(
            &s_output_list,
            "get",
            "(I)Ljava/lang/Object;",
            &[JValue::Int(i as jint)],
        ) {
            Ok(val) => match val.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if item.is_null() {
            continue;
        }

        // Get address string
        let address_field = match env.get_field(&item, "address", "Ljava/lang/String;") {
            Ok(field) => match field.l() {
                Ok(obj) => obj,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if address_field.is_null() {
            continue;
        }

        let j_string = JString::from(address_field);
        let r_string = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_str = match r_string.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let address_cstr = match CString::new(address_str) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Get value
        let value = match env.get_field(&item, "value", "J") {
            Ok(field) => field.j().unwrap_or(0) as u64,
            Err(_) => continue,
        };

        // Get memo_type
        let memo_type = match env.get_field(&item, "memoType", "B") {
            Ok(field) => field.b().unwrap_or(0) as u8,
            Err(_) => 0,
        };

        // Get has_ovk
        let has_ovk = match env.get_field(&item, "hasOvk", "Z") {
            Ok(field) => field.z().unwrap_or(false),
            Err(_) => false,
        };

        // Process OVK if present
        let (ovk_ptr, ovk_len) = if has_ovk {
            let ovk_obj = match env.get_field(&item, "ovk", "[B") {
                Ok(field) => match field.l() {
                    Ok(obj) => obj,
                    Err(_) => JObject::null(),
                },
                Err(_) => JObject::null(),
            };

            if ovk_obj.is_null() {
                (std::ptr::null(), 0)
            } else {
                // Cast to JByteArray so that it implements AsJArrayRaw
                let ovk_array = JByteArray::from(ovk_obj);
                let ovk_array_len = match env.get_array_length(&ovk_array) {
                    Ok(len) => len as usize,
                    Err(_) => 0,
                };

                if ovk_array_len != 32 {
                    (std::ptr::null(), 0)
                } else {
                    let mut ovk_bytes = vec![0i8; 32];
                    if let Err(_) = env.get_byte_array_region(&ovk_array, 0, &mut ovk_bytes) {
                        (std::ptr::null(), 0)
                    } else {
                        // Convert to u8 vector
                        let ovk_u8: Vec<u8> = ovk_bytes.iter().map(|&x| x as u8).collect();
                        ovk_storage.push(ovk_u8);
                        (ovk_storage.last().unwrap().as_ptr(), 32)
                    }
                }
            }
        } else {
            (std::ptr::null(), 0)
        };

        address_storage.push(address_cstr);

        s_output_vec.push(CSaplingOutData {
            address: address_storage.last().unwrap().as_ptr(),
            value,
            memo_type,
            has_ovk,
            ovk: ovk_ptr,
            ovk_len,
        });
    }

    // Create the CInitData struct
    let init_data = CInitData {
        t_in: if t_in_vec.is_empty() {
            std::ptr::null()
        } else {
            t_in_vec.as_ptr()
        },
        t_in_len: t_in_vec.len(),
        t_out: if t_out_vec.is_empty() {
            std::ptr::null()
        } else {
            t_out_vec.as_ptr()
        },
        t_out_len: t_out_vec.len(),
        s_spend: if s_spend_vec.is_empty() {
            std::ptr::null()
        } else {
            s_spend_vec.as_ptr()
        },
        s_spend_len: s_spend_vec.len(),
        s_output: if s_output_vec.is_empty() {
            std::ptr::null()
        } else {
            s_output_vec.as_ptr()
        },
        s_output_len: s_output_vec.len(),
    };

    // Prepare parameters for the C function
    let mut result_ptr: *mut u8 = std::ptr::null_mut();
    let mut result_len: u64 = 0;

    // Call the C-exported function
    let error_code = get_inittx_data(init_data, &mut result_ptr, &mut result_len);

    if error_code != ZcashError::Success as u32 {
        error!("Error in get_inittx_data: error code {}", error_code);
        return std::ptr::null_mut();
    }

    if result_ptr.is_null() || result_len == 0 {
        error!("get_inittx_data returned null pointer or zero length");
        return std::ptr::null_mut();
    }

    // Create a Java byte array for the result
    let byte_array = match env.new_byte_array(result_len as jsize) {
        Ok(array) => array,
        Err(e) => {
            error!("Failed to create byte array: {:?}", e);
            free_transaction_data(result_ptr);
            return std::ptr::null_mut();
        }
    };

    // Copy the result data (converting u8 to i8 for JNI)
    let bytes: Vec<i8> = std::slice::from_raw_parts(result_ptr, result_len as usize)
        .iter()
        .map(|&b| b as i8)
        .collect();

    if let Err(e) = env.set_byte_array_region(&byte_array, 0, &bytes) {
        error!("Failed to set byte array region: {:?}", e);
        free_transaction_data(result_ptr);
        return std::ptr::null_mut();
    }

    free_transaction_data(result_ptr);
    byte_array.into_raw()
}
