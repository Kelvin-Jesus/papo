//! The profile directory is the only state papo keeps; these tests pin down how it
//! behaves on disk, including when files are damaged.

use std::{collections::BTreeMap, fs, path::Path};

use iroh::SecretKey;
use papo::{
    proto::{Envelope, PeerKind},
    room::RoomSecret,
    store::{KnownPeer, LogEntry, Profile, Store, validate_profile_name},
};

fn profile(name: &str) -> Profile {
    Profile { name: name.into(), about: Some("api".into()), room: RoomSecret::generate().to_base32(), created_ms: 1 }
}

fn new_store(dir: &Path) -> Store {
    Store::create_at(dir.join("p"), &profile("kj")).unwrap()
}

fn env(id: &str) -> Envelope {
    Envelope {
        id: id.into(),
        from: "ana".into(),
        node: "n".into(),
        kind: PeerKind::Agent,
        to: None,
        reply_to: None,
        ts: 1,
        body: format!("corpo {id} com acentuação e emoji 🐦"),
    }
}

#[test]
fn profile_roundtrips_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    let reopened = Store::open_at(store.dir().to_path_buf()).unwrap();
    let p = reopened.profile().unwrap();
    assert_eq!((p.name.as_str(), p.about.as_deref()), ("kj", Some("api")));
    assert!(p.room_secret().is_ok());

    let mut changed = p.clone();
    changed.about = None;
    reopened.save_profile(&changed).unwrap();
    assert_eq!(reopened.profile().unwrap().about, None);
}

#[test]
fn opening_a_missing_profile_fails_with_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let err = Store::open_at(dir.path().join("nothing")).unwrap_err();
    assert!(err.to_string().contains("no profile"), "{err}");
}

#[test]
fn invalid_member_names_are_refused_at_creation() {
    let dir = tempfile::tempdir().unwrap();
    assert!(Store::create_at(dir.path().join("p"), &profile("tem espaço")).is_err());
    assert!(!dir.path().join("p").exists(), "nothing is written for an invalid profile");
}

#[test]
fn identity_is_generated_once_and_stays_stable() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    let a = store.secret_key().unwrap();
    let b = store.secret_key().unwrap();
    assert_eq!(a.public(), b.public());

    // Recreating the profile (new room) keeps the identity peers already know.
    let again = Store::create_at(store.dir().to_path_buf(), &profile("kj")).unwrap();
    assert_eq!(again.secret_key().unwrap().public(), a.public());
}

#[test]
fn recreating_a_profile_drops_the_old_rooms_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    store.save_inbox(&[env("a")]).unwrap();
    store.save_outbox(&[env("b")]).unwrap();
    store
        .save_known_peers(&[(SecretKey::generate().public(), KnownPeer::default())].into_iter().collect())
        .unwrap();

    let again = Store::create_at(store.dir().to_path_buf(), &profile("kj")).unwrap();
    assert!(again.inbox().unwrap().is_empty());
    assert!(again.outbox().unwrap().is_empty());
    assert!(again.known_peers().unwrap().is_empty());
}

#[test]
fn corrupt_identity_is_reported_not_silently_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    fs::write(store.dir().join("secret.key"), "not hex").unwrap();
    assert!(store.secret_key().unwrap_err().to_string().contains("corrupt"));
    fs::write(store.dir().join("secret.key"), "abcd").unwrap();
    assert!(store.secret_key().unwrap_err().to_string().contains("wrong length"));
}

#[test]
fn corrupt_room_secret_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    let mut p = store.profile().unwrap();
    p.room = "!!!".into();
    store.save_profile(&p).unwrap();
    assert!(store.profile().unwrap().room_secret().unwrap_err().to_string().contains("corrupt room secret"));
}

#[test]
fn inbox_and_outbox_roundtrip_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    assert!(store.inbox().unwrap().is_empty(), "missing file means empty");
    let msgs = vec![env("a"), env("b")];
    store.save_inbox(&msgs).unwrap();
    store.save_outbox(&msgs[..1]).unwrap();
    assert_eq!(store.inbox().unwrap(), msgs);
    assert_eq!(store.outbox().unwrap(), msgs[..1].to_vec());
    // Write-then-rename leaves no temporary files behind.
    let leftovers: Vec<_> = fs::read_dir(store.dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn corrupt_inbox_is_an_error_naming_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    fs::write(store.dir().join("inbox.json"), "{oops").unwrap();
    let err = format!("{:#}", store.inbox().unwrap_err());
    assert!(err.contains("inbox.json"), "{err}");
}

#[test]
fn known_peers_skip_entries_that_do_not_parse() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    let good = SecretKey::generate().public();
    let mut peers = BTreeMap::new();
    peers.insert(good, KnownPeer { name: Some("bob".into()), last_seen_ms: 42 });
    store.save_known_peers(&peers).unwrap();

    // Someone hand-edits the file and leaves a bad id in it.
    let path = store.dir().join("peers.json");
    let mut raw: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    raw["not-an-endpoint-id"] = serde_json::json!({"name": "x", "last_seen_ms": 1});
    fs::write(&path, raw.to_string()).unwrap();

    let loaded = store.known_peers().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[&good], KnownPeer { name: Some("bob".into()), last_seen_ms: 42 });
}

#[test]
fn log_tolerates_torn_and_garbage_lines() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    assert!(store.read_log().unwrap().is_empty());
    store.append_log(&LogEntry::In { msg: env("a") }).unwrap();
    store.append_log(&LogEntry::Delivered { id: "a".into(), by: "bob".into(), ts: 5 }).unwrap();
    let mut file = fs::OpenOptions::new().append(true).open(store.log_path()).unwrap();
    std::io::Write::write_all(&mut file, b"garbage line\n{\"ev\":\"out\",\"msg\":").unwrap();
    let log = store.read_log().unwrap();
    assert_eq!(log.len(), 2, "valid entries survive a crash mid-write");
    assert!(matches!(&log[1], LogEntry::Delivered { id, .. } if id == "a"));
}

#[test]
fn concurrent_appends_never_interleave_lines() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    std::thread::scope(|s| {
        for t in 0..8 {
            let store = store.clone();
            s.spawn(move || {
                for i in 0..50 {
                    store.append_log(&LogEntry::In { msg: env(&format!("{t}-{i}")) }).unwrap();
                }
            });
        }
    });
    let raw = fs::read_to_string(store.log_path()).unwrap();
    assert_eq!(raw.lines().count(), 400);
    assert_eq!(store.read_log().unwrap().len(), 400, "every line is a whole JSON entry");
}

#[test]
fn only_one_holder_of_the_profile_lock() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    let held = store.lock().unwrap();
    let err = store.lock().unwrap_err();
    assert!(err.to_string().contains("already running"), "{err}");
    drop(held);
    assert!(store.lock().is_ok(), "the lock is released with the file");
}

#[cfg(unix)]
#[test]
fn secrets_are_private_to_the_user() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(dir.path());
    store.secret_key().unwrap();
    let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(store.dir()), 0o700);
    assert_eq!(mode(&store.dir().join("profile.json")), 0o600);
    assert_eq!(mode(&store.dir().join("secret.key")), 0o600);
    // Rewriting keeps the permissions (atomic replace creates a new file).
    store.save_profile(&store.profile().unwrap()).unwrap();
    assert_eq!(mode(&store.dir().join("profile.json")), 0o600);
}

#[test]
fn profile_names_are_restricted_to_safe_directory_names() {
    for ok in ["default", "time-b", "a_1", &"x".repeat(32)] {
        assert!(validate_profile_name(ok).is_ok(), "{ok}");
    }
    for bad in ["", "../etc", "a b", "a/b", "ação", &"x".repeat(33)] {
        assert!(validate_profile_name(bad).is_err(), "{bad}");
    }
    assert!(Store::dir_for("../escape").is_err());
}
