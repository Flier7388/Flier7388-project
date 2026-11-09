# "Unable to decrypt" placeholder (2026-11-09, fifth phase of the day)

## What changed
Owner noticed that undecryptable messages weren't just unreadable —
they were completely invisible, indistinguishable from missing history.
`crates/matrix-client/src/timeline.rs`'s `as_timeline_message` (renamed
from `as_text_message`) previously only matched `MsgLikeKind::Message`
and silently returned `None` for everything else, including
`MsgLikeKind::UnableToDecrypt`.

Now `UnableToDecrypt` events produce a real `TimelineMessage` too, with
`body` set to a placeholder: `"🔒 Unable to decrypt message (<reason>)"`
for the common `MegolmV1AesSha2` case, with the reason coming from
`matrix-sdk-crypto`'s own `UtdCause` (added `matrix-sdk-base` as a
direct dependency with `e2e-encryption` specifically to reach this type
— `matrix-sdk` doesn't re-export it). Looked at `git/robrix`'s
`event_preview.rs` for the real-world cause→text mapping pattern before
writing a simplified version of it (fewer, shorter reasons — this
project doesn't need robrix's full granularity).

No bridge or Dart changes needed at all — `TimelineMessage`'s shape
(`sender`/`body`, both plain strings) didn't change, so the existing
rendering just shows the placeholder text like any other message.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge`, `flutter
build linux` — all clean (no `dart analyze` needed, no Dart files
touched). **Owner confirmed live**: after rebuilding, one message from
real room history now shows the placeholder where it was previously
just absent.

## Still open
- Only `MegolmV1AesSha2`'s `cause` gets a specific reason string; the
  rarer `OlmV1Curve25519AesSha2`/`Unknown` encryption-metadata variants
  just get the generic placeholder with no reason. Fine for now — those
  are rare in practice (robrix's own cause breakdown is also keyed off
  `MegolmV1AesSha2` specifically).
- No retry-on-key-arrival: if the key shows up later (e.g. via key
  backup or a late-arriving to-device message), the placeholder won't
  automatically turn into real text without a fresh `subscribe_messages`
  read. `Timeline`'s own diff stream should in principle re-emit the
  item once the content updates — not independently verified this
  session.
