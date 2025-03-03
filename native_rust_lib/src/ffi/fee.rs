#[no_mangle]
pub extern "C" fn calculate_fee(n_tin: usize, n_tout: usize, n_spend: usize, n_sout: usize) -> u64 {
    ledger_app_builder::builder::Builder::calculate_zip0317_fee(n_tin, n_tout, n_spend, n_sout)
        .into()
}
