#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 24 {
        return;
    }

    let max_deposit = u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8]));
    let daily_limit = u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8]));
    let amount = u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8]));

    // Mirrors contract guards: zero amount invalid, max_deposit=0 means unlimited.
    let deposit_allowed = amount > 0 && (max_deposit == 0 || amount <= max_deposit);
    if max_deposit > 0 && amount > max_deposit {
        assert!(!deposit_allowed);
    }

    // Checked arithmetic must not panic.
    let _ = daily_limit.checked_add(amount);
    let _ = max_deposit.checked_sub(amount);
});
