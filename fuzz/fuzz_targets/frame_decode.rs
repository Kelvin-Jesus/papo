//! A room member controls the plaintext of every frame it seals. Seal the fuzzer's
//! bytes with a fixed room key so libFuzzer explores the JSON frame parser itself, and
//! check that every frame that parses survives the wire again.
#![no_main]

use libfuzzer_sys::fuzz_target;
use papo::{proto, room::RoomSecret};

fuzz_target!(|data: &[u8]| {
    // Fixed key: the fuzzer needs reproducible inputs.
    let room = RoomSecret::from_base32("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let _ = proto::decode(&room, data);
    if let Ok(frame) = proto::decode(&room, &room.seal(data))
        && let Ok(bytes) = proto::encode(&room, &frame)
    {
        assert_eq!(proto::decode(&room, &bytes).expect("re-encoded frame must decode"), frame);
    }
});
