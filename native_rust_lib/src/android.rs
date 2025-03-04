use jni::{sys::jstring, JNIEnv};

mod builder;
pub use builder::*;
mod fee;
pub use fee::*;
mod init_data;
pub use init_data::*;
mod error;
pub use error::*;

pub(crate) fn init_logger() {
    use android_logger::Config;
    use log::LevelFilter;

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("RustModule"),
    );
}

#[cfg(any(target_os = "android", test))]
pub fn create_java_string(env: &JNIEnv, desc_str: &str) -> jstring {
    // Fix: Use into_raw() instead of into_inner()
    env.new_string(desc_str).unwrap().into_raw()
}

#[cfg(test)]
mod android_tx_test {

    use crate::{
        test::{get_or_init_jvm, open_sapling_params, open_test_data},
        ZcashError,
    };

    use super::*;
    use jni::{
        objects::{JByteArray, JClass, JObject, JString, JValue},
        InitArgsBuilder, JNIEnv, JavaVM,
    };

    const TEST_NAME: &str = "tests/tx_2ti_2to.json";
    const TX_VERSION: u8 = 5;

    #[test]
    fn test_complete_transaction_flow() {
        let jvm = get_or_init_jvm();

        // Attach to the current thread
        let mut env = jvm
            .attach_current_thread()
            .expect("Failed to attach thread");

        // Get raw JNIEnv pointer for C functions
        let env_ptr = env.get_native_interface();

        // Load test data
        let test_data = open_test_data(TEST_NAME);
        println!("Test data loaded successfully");

        // Step 1: Calculate fee
        let t_in_count = test_data.num_tinputs() as i32;
        let t_out_count = test_data.num_toutputs() as i32;
        let s_spend_count = test_data.num_sspends() as i32;
        let s_output_count = test_data.num_soutputs() as i32;

        let fee = unsafe {
            Java_expo_modules_myrustmodule_MyRustModule_calculateFee(
                JNIEnv::from_raw(env_ptr).unwrap(),
                JClass::from(JObject::null()),
                t_in_count,
                t_out_count,
                s_spend_count,
                s_output_count,
            )
        };

        println!("Calculated fee: {}", fee);

        // Step 2: Initialize transaction data
        // ===== Create InitData object =====
        let init_data = &test_data.tx_init_data;

        // Create an empty InitData object
        let init_obj = env
            .new_object("expo/modules/myrustmodule/InitData", "()V", &[])
            .expect("Failed to create InitData object");

        // Create ArrayLists for each data type
        let t_in_list = env
            .new_object("java/util/ArrayList", "()V", &[])
            .expect("Failed to create t_in list");
        let t_out_list = env
            .new_object("java/util/ArrayList", "()V", &[])
            .expect("Failed to create t_out list");
        let s_spend_list = env
            .new_object("java/util/ArrayList", "()V", &[])
            .expect("Failed to create s_spend list");
        let s_output_list = env
            .new_object("java/util/ArrayList", "()V", &[])
            .expect("Failed to create s_output list");

        // Add each transparent input
        for tin in &init_data.t_in {
            let tin_obj = env
                .new_object("expo/modules/myrustmodule/TinData", "()V", &[])
                .expect("Failed to create TinData object");

            // Set path
            let path: Vec<i64> = tin.path.iter().map(|&x| x as i64).collect();
            let path_array = env
                .new_long_array(tin.path.len() as _)
                .expect("Failed to create path array");
            env.set_long_array_region(&path_array, 0, &path)
                .expect("Failed to set path array region");
            env.set_field(&tin_obj, "path", "[J", JValue::Object(&path_array))
                .expect("Failed to set path field");

            // Set address
            let jstr = env
                .new_string(tin.address.to_str().unwrap())
                .expect("Failed to create address string");
            env.set_field(
                &tin_obj,
                "address",
                "Ljava/lang/String;",
                JValue::Object(&jstr),
            )
            .expect("Failed to set address field");

            // Set value
            env.set_field(&tin_obj, "value", "J", JValue::Long(tin.value as _))
                .expect("Failed to set value field");

            // Add to list
            env.call_method(
                &t_in_list,
                "add",
                "(Ljava/lang/Object;)Z",
                &[JValue::Object(&tin_obj)],
            )
            .expect("Failed to add TinData to list");
        }

        // Add each transparent output
        for tout in &init_data.t_out {
            let tout_obj = env
                .new_object("expo/modules/myrustmodule/ToutData", "()V", &[])
                .expect("Failed to create ToutData object");

            // Set address
            let jstr = env
                .new_string(tout.address.to_str().unwrap())
                .expect("Failed to create address string");
            env.set_field(
                &tout_obj,
                "address",
                "Ljava/lang/String;",
                JValue::Object(&jstr),
            )
            .expect("Failed to set address field");

            // Set value
            env.set_field(&tout_obj, "value", "J", JValue::Long(tout.value as _))
                .expect("Failed to set value field");

            // Add to list
            env.call_method(
                &t_out_list,
                "add",
                "(Ljava/lang/Object;)Z",
                &[JValue::Object(&tout_obj)],
            )
            .expect("Failed to add ToutData to list");
        }

        // Set the lists as fields in the InitData object
        env.set_field(
            &init_obj,
            "tIn",
            "Ljava/util/List;",
            JValue::Object(&t_in_list),
        )
        .expect("Failed to set tIn field");
        env.set_field(
            &init_obj,
            "tOut",
            "Ljava/util/List;",
            JValue::Object(&t_out_list),
        )
        .expect("Failed to set tOut field");
        env.set_field(
            &init_obj,
            "sSpend",
            "Ljava/util/List;",
            JValue::Object(&s_spend_list),
        )
        .expect("Failed to set sSpend field");
        env.set_field(
            &init_obj,
            "sOutput",
            "Ljava/util/List;",
            JValue::Object(&s_output_list),
        )
        .expect("Failed to set sOutput field");

        // Call getInitTxData
        let init_tx_data = unsafe {
            Java_expo_modules_myrustmodule_MyRustModule_getInitTxData(
                JNIEnv::from_raw(env_ptr).unwrap(),
                JClass::from(JObject::null()),
                init_obj,
            )
        };

        // Verify init_tx_data against expected value
        assert!(!init_tx_data.is_null(), "Init TX data should not be null");

        let byte_array = unsafe { JByteArray::from_raw(init_tx_data) };
        let len = env.get_array_length(&byte_array).unwrap();
        assert!(len > 0, "Result array should not be empty");

        // Get the bytes from the array for comparison
        let mut bytes = vec![0i8; len as usize];
        env.get_byte_array_region(&byte_array, 0, &mut bytes)
            .unwrap();

        // Convert to unsigned bytes for comparison
        let u_bytes: Vec<u8> = bytes.iter().map(|&b| b as u8).collect();

        // Print as hex string for comparison
        let hex_str = u_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");

        println!("Init TX data hex: {}", hex_str);

        // Compare with expected hex
        assert_eq!(
            hex_str,
            test_data.init_data_blob(),
            "Init TX data does not match expected value"
        );

        // IMPORTANT to: Clean up
        env.delete_local_ref(byte_array).unwrap();

        // Step 3: Create transaction builder (using the fee we calculated)
        let network_type = 1; // Assuming 1 is for testnet, adjust as needed
        let builder_id = unsafe {
            Java_expo_modules_myrustmodule_MyRustModule_createBuilder(
                JNIEnv::from_raw(env_ptr).unwrap(),
                JClass::from(JObject::null()),
                fee,
                network_type,
            )
        };

        println!("Created builder with ID: {}", builder_id as u32);

        // Step 4: Add transparent inputs
        for (idx, t_input) in test_data.tinput_iter().enumerate() {
            println!("adding tinput: {:?}", t_input);
            // Create TransparentInput object
            let input_obj = env
                .new_object("expo/modules/myrustmodule/TransparentInput", "()V", &[])
                .expect("Failed to create TransparentInput object");

            // Set outpoint
            let outp_str = env
                .new_string(t_input.outp_str())
                .expect("Failed to create outpoint string");
            env.set_field(
                &input_obj,
                "outp",
                "Ljava/lang/String;",
                JValue::Object(&outp_str),
            )
            .expect("Failed to set outpoint field");

            // Set public key
            let pk_str = env
                .new_string(t_input.pk_str())
                .expect("Failed to create public key string");
            env.set_field(
                &input_obj,
                "pk",
                "Ljava/lang/String;",
                JValue::Object(&pk_str),
            )
            .expect("Failed to set publicKey field");

            // Set address
            let addr_str = env
                .new_string(t_input.address_str())
                .expect("Failed to create address string");
            env.set_field(
                &input_obj,
                "address",
                "Ljava/lang/String;",
                JValue::Object(&addr_str),
            )
            .expect("Failed to set address field");

            // Set value
            env.set_field(&input_obj, "value", "J", JValue::Long(t_input.value as i64))
                .expect("Failed to set value field");

            // Call addTransparentInput
            let result = unsafe {
                Java_expo_modules_myrustmodule_MyRustModule_addTransparentInput(
                    JNIEnv::from_raw(env_ptr).unwrap(),
                    JClass::from(JObject::null()),
                    builder_id,
                    input_obj,
                )
            };

            println!("Added transparent input {}: result = {}", idx, result);
            assert_eq!(
                result,
                ZcashError::Success as i32,
                "Adding transparent input {} should succeed",
                idx
            );
        }

        // Step 5: Add transparent outputs
        for (idx, t_output) in test_data.toutput_iter().enumerate() {
            // Create TransparentOutput object
            let output_obj = env
                .new_object("expo/modules/myrustmodule/TransparentOutput", "()V", &[])
                .expect("Failed to create TransparentOutput object");

            // Set address
            let addr_str = env
                .new_string(t_output.address_str())
                .expect("Failed to create address string");
            env.set_field(
                &output_obj,
                "address",
                "Ljava/lang/String;",
                JValue::Object(&addr_str),
            )
            .expect("Failed to set address field");

            // Set value
            env.set_field(
                &output_obj,
                "value",
                "J",
                JValue::Long(t_output.value as i64),
            )
            .expect("Failed to set value field");

            // Call addTransparentOutput
            let result = unsafe {
                Java_expo_modules_myrustmodule_MyRustModule_addTransparentOutput(
                    JNIEnv::from_raw(env_ptr).unwrap(),
                    JClass::from(JObject::null()),
                    builder_id,
                    output_obj,
                )
            };

            println!("Added transparent output {}: result = {}", idx, result);
            assert_eq!(
                result,
                ZcashError::Success as i32,
                "Adding transparent output {} should succeed",
                idx
            );
        }

        // Step 6: Build transaction
        // Empty strings for spend_path and output_path as they're not used in this test
        let (spend_path, output_path) = open_sapling_params();
        let spend_path = env.new_string(&spend_path).unwrap();
        let output_path = env.new_string(&output_path).unwrap();

        let tx_bytes = unsafe {
            Java_expo_modules_myrustmodule_MyRustModule_buildTransaction(
                JNIEnv::from_raw(env_ptr).unwrap(),
                JClass::from(JObject::null()),
                builder_id,
                spend_path,
                output_path,
                TX_VERSION as _,
            )
        };

        // We don't have expected value for the built transaction in the test data
        // Just verify it's not null
        assert!(!tx_bytes.is_null(), "Built transaction should not be null");

        let tx_byte_array = unsafe { JByteArray::from_raw(tx_bytes) };
        let tx_len = env.get_array_length(&tx_byte_array).unwrap();
        println!("Built transaction size: {} bytes", tx_len);
        assert!(tx_len > 0, "Built transaction should not be empty");

        // Clean up
        env.delete_local_ref(tx_byte_array).unwrap();

        // Step 7: Add signatures
        // Create Signatures object
        let signatures_obj = env
            .new_object("expo/modules/myrustmodule/Signatures", "()V", &[])
            .expect("Failed to create Signatures object");

        // Create ArrayList for transparent signatures
        let t_sigs_list = env
            .new_object("java/util/ArrayList", "()V", &[])
            .expect("Failed to create transparent signatures list");

        // Add each transparent signature
        for sig in test_data.transparent_sig() {
            let sig_str = env
                .new_string(sig.to_str().unwrap())
                .expect("Failed to create signature string");
            env.call_method(
                &t_sigs_list,
                "add",
                "(Ljava/lang/Object;)Z",
                &[JValue::Object(&sig_str)],
            )
            .expect("Failed to add signature to list");
        }

        // Create ArrayList for sapling signatures (empty in this test)
        let s_sigs_list = env
            .new_object("java/util/ArrayList", "()V", &[])
            .expect("Failed to create sapling signatures list");

        // Set the lists as fields in the Signatures object
        env.set_field(
            &signatures_obj,
            "transparentSigs",
            "Ljava/util/List;",
            JValue::Object(&t_sigs_list),
        )
        .expect("Failed to set transparentSigs field");
        env.set_field(
            &signatures_obj,
            "saplingSigs",
            "Ljava/util/List;",
            JValue::Object(&s_sigs_list),
        )
        .expect("Failed to set saplingSigs field");

        // Call addSignatures
        let result = unsafe {
            Java_expo_modules_myrustmodule_MyRustModule_addSignatures(
                JNIEnv::from_raw(env_ptr).unwrap(),
                JClass::from(JObject::null()),
                builder_id,
                signatures_obj,
            )
        };

        println!("Added signatures: result = {}", result);
        assert_eq!(
            result,
            ZcashError::Success as i32,
            "Adding signatures should succeed"
        );

        // Step 8: Finalize transaction
        let final_tx = unsafe {
            Java_expo_modules_myrustmodule_MyRustModule_finalizeTransaction(
                JNIEnv::from_raw(env_ptr).unwrap(),
                JClass::from(JObject::null()),
                builder_id,
            )
        };

        // Verify final transaction against expected value
        assert!(
            !final_tx.is_null(),
            "Finalized transaction should not be null"
        );

        let final_byte_array = unsafe { JByteArray::from_raw(final_tx) };
        let final_len = env.get_array_length(&final_byte_array).unwrap();
        assert!(final_len > 0, "Finalized transaction should not be empty");

        let mut final_bytes = vec![0i8; final_len as usize];
        env.get_byte_array_region(&final_byte_array, 0, &mut final_bytes)
            .unwrap();

        // Convert to unsigned bytes for comparison
        let final_u_bytes: Vec<u8> = final_bytes.iter().map(|&b| b as u8).collect();

        // Print as hex string for comparison
        let final_hex_str = final_u_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");

        println!("Finalized transaction hex: {}", final_hex_str);

        // Compare with expected hex
        assert_eq!(
            final_hex_str,
            test_data.finalized_tx_blob(),
            "Finalized transaction does not match expected value"
        );

        // Clean up
        env.delete_local_ref(final_byte_array).unwrap();

        println!("Transaction flow test completed successfully!");
    }
}
