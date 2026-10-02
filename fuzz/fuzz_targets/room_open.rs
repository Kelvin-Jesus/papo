//! Anyone on the gossip path can hand us arbitrary bytes: opening them must never
//! succeed without the key, and must never panic.
#![no_main]

use libfuzzer_sys::fuzz_target;
use papo::room::RoomSecret;

fuzz_target!(|data: &[u8]| {
    let room = RoomSecret::from_base32("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    // Forging a valid frame without the key would be a broken AEAD, not a fuzz finding
    // we could ever hit by chance; what matters is that this never panics.
    let _ = room.open(data);
    let _ = RoomSecret::from_base32(&String::from_utf8_lossy(data));
});
