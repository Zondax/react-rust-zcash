use crate::{calculate_fee, init_logger};

use jni::{
    objects::JClass,
    sys::{jint, jlong},
    JNIEnv,
};

use log::info;

#[no_mangle]
pub unsafe extern "C" fn Java_expo_modules_myrustmodule_MyRustModule_calculateFee(
    mut env: JNIEnv,
    _class: JClass,
    a: jint,
    b: jint,
    c: jint,
    d: jint,
) -> jlong {
    init_logger();

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
