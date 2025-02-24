mod builder;
mod error;
mod network;
mod parser;
mod transparent_input;
mod transparent_output;

pub use error::ZcashError;
pub use network::NetworkType;
pub use transparent_input::TransparentInputInfo;
pub use transparent_output::TransparentOutputInfo;
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
    use android_logger::Config;
    use jni::sys::{jint, jlong};
    use jni::JNIEnv;
    use log::info;
    use log::LevelFilter;

    use crate::calculate_fee;
    use jni::objects::JClass;

    #[no_mangle]
    pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_calculateFee(
        mut env: JNIEnv,
        _class: JClass,
        a: jint,
        b: jint,
        c: jint,
        d: jint,
    ) -> jlong {
        android_logger::init_once(
            Config::default()
                .with_max_level(LevelFilter::Info)
                .with_tag("RustModule"),
        );

        info!("calculateFee called with inputs: {} {} {} {}", a, b, c, d);

        if a < 0 || b < 0 || c < 0 || d < 0 {
            info!("Negative input values detected");
            let exception_class = env
                .find_class("java/lang/IllegalArgumentException")
                .unwrap();
            env.throw_new(exception_class, "Negative values are not allowed")
                .unwrap();
            return 0;
        }

        let n_tin = a as usize;
        let n_tout = b as usize;
        let n_spend = c as usize;
        let n_sout = d as usize;

        let result = calculate_fee(n_tin, n_tout, n_spend, n_sout);
        info!("Fee calculation result: {}", result);

        match result {
            fee if fee <= i64::MAX as u64 => fee as jlong,
            _ => {
                info!("Fee calculation overflow");
                let exception_class = env.find_class("java/lang/ArithmeticException").unwrap();
                env.throw_new(exception_class, "Fee calculation resulted in overflow")
                    .unwrap();
                0
            }
        }
    }
}
