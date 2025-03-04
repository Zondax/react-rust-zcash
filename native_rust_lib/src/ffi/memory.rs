use std::ffi::{c_char, CString};

#[no_mangle]
pub extern "C" fn free_transaction_data(data: *mut u8) {
    if !data.is_null() {
        unsafe {
            libc::free(data as *mut libc::c_void);
        }
    }
}

// Helper function to free error description memory
#[no_mangle]
pub extern "C" fn free_error_description(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
