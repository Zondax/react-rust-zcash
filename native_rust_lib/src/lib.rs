pub type Amount = u64;

#[no_mangle]
pub extern "C" fn calculate_fee(
    n_tin: usize,
    n_tout: usize,
    n_spend: usize,
    n_sout: usize,
) -> Amount {
    ledger_app_builder::builder::Builder::calculate_zip0317_fee(n_tin, n_tout, n_spend, n_sout)
        .into()
}

/// cbindgen:ignore
#[cfg(target_os = "android")]
pub mod android {
    use crate::calculate_fee;
    use jni::objects::JClass;
    use jni::sys::jint;
    use jni::JNIEnv;

    #[no_mangle]
    pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_calculateFee(
        _env: JNIEnv,
        _class: JClass,
        a: jint,
        b: jint,
        c: jint,
        d: jint,
    ) -> jint {
        let n_tin = a as usize;
        let n_tout = b as usize;
        let n_spend = c as usize;
        let n_sout = d as usize;
        calculate_fee(n_tin, n_tout, n_spend, n_sout) as jint
    }
}
