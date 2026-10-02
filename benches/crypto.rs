//! Hot paths of every message: sealing/opening frames, JSON framing and invites.
//!
//! `cargo bench` runs them; CI only compiles them (`cargo bench --no-run`) because
//! shared runners are too noisy for meaningful numbers.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use iroh::SecretKey;
use papo::{
    proto::{self, Envelope, Frame, PeerKind},
    room::{Invite, RoomSecret},
};

fn envelope(body_len: usize) -> Envelope {
    Envelope {
        id: proto::new_msg_id(),
        from: "kj".into(),
        node: SecretKey::generate().public().to_string(),
        kind: PeerKind::Agent,
        to: Some("ana".into()),
        reply_to: None,
        ts: proto::now_ms(),
        body: "x".repeat(body_len),
    }
}

fn seal_open(c: &mut Criterion) {
    let room = RoomSecret::generate();
    let mut group = c.benchmark_group("seal_open");
    // A one-line question, a typical code snippet, and the maximum body.
    for size in [128usize, 4 * 1024, proto::MAX_BODY_BYTES] {
        let plaintext = vec![b'a'; size];
        let sealed = room.seal(&plaintext);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("seal", size), &plaintext, |b, p| b.iter(|| room.seal(black_box(p))));
        group.bench_with_input(BenchmarkId::new("open", size), &sealed, |b, s| {
            b.iter(|| room.open(black_box(s)).unwrap())
        });
    }
    group.finish();
}

fn frames(c: &mut Criterion) {
    let room = RoomSecret::generate();
    let mut group = c.benchmark_group("frame");
    for size in [128usize, 4 * 1024] {
        let frame = Frame::Msg(envelope(size));
        let bytes = proto::encode(&room, &frame).unwrap();
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_with_input(BenchmarkId::new("encode", size), &frame, |b, f| {
            b.iter(|| proto::encode(&room, black_box(f)).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("decode", size), &bytes, |b, bytes| {
            b.iter(|| proto::decode(&room, black_box(bytes)).unwrap())
        });
    }
    group.finish();
}

fn invites(c: &mut Criterion) {
    let invite =
        Invite { secret: RoomSecret::generate(), peers: (0..4).map(|_| SecretKey::generate().public()).collect() };
    let code = invite.encode();
    c.bench_function("invite/encode", |b| b.iter(|| black_box(&invite).encode()));
    c.bench_function("invite/decode", |b| b.iter(|| Invite::decode(black_box(&code)).unwrap()));
    let room = RoomSecret::generate();
    c.bench_function("room/derive_topic", |b| b.iter(|| black_box(&room).topic()));
}

criterion_group!(benches, seal_open, frames, invites);
criterion_main!(benches);
