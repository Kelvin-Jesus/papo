//! Invites arrive by copy-paste from chat apps: any text must decode or fail cleanly,
//! and whatever decodes must re-encode to an equivalent invite.
#![no_main]

use libfuzzer_sys::fuzz_target;
use papo::room::Invite;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    for candidate in [text.to_string(), format!("papo1{text}")] {
        if let Ok(invite) = Invite::decode(&candidate) {
            let again = Invite::decode(&invite.encode()).expect("re-encoded invite must decode");
            assert_eq!(again, invite);
        }
    }
});
