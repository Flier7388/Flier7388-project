//! Android process entry point.
//!
//! `android-activity`'s `native-activity` backend requires an exported
//! `android_main(app: AndroidApp)` function as its sole entry point —
//! verified directly against the `android-activity` 0.6.1 crate's own
//! rustdoc example, which shows exactly the signature and attribute
//! used below. `iced_winit`'s Android support (added in
//! <https://github.com/iced-rs/iced/pull/3245>, not yet on crates.io —
//! see `crates/app-ui/Cargo.toml`) expects the resulting `AndroidApp`
//! handle to be stashed in its `ANDROID_APP` static *before*
//! [`crate::run`] is called, so that `iced_winit::run` can attach the
//! `winit` event loop to this Android activity via
//! `EventLoopBuilderExtAndroid::with_android_app`.

use android_activity::AndroidApp;

/// Android process entry point, called by the `android-activity`
/// native glue once the `Activity` has loaded (and potentially called
/// again across the `Activity`'s lifetime — see the note below).
///
/// # Safety-lint note
///
/// `#[unsafe(no_mangle)]` is required so the dynamic linker can find
/// this *exact* symbol name; `android-activity` documents this as the
/// only supported entry point shape. This is the one, deliberate
/// exception to the workspace's `unsafe_code = "deny"` lint in this
/// crate — every other `app-ui` module stays entirely safe Rust.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // `android-activity`'s own docs note that `android_main` "is
    // possible... to run multiple times over the life of the
    // process" (e.g. Activity recreation). We deliberately do not
    // `.expect()` on the second call: a panic here would crash the
    // whole process, whereas silently keeping the first `AndroidApp`
    // handle at least keeps the app alive, if possibly stale.
    // Whether that staleness is actually observable (frozen UI after
    // a configuration change) is exactly what the manual rotation /
    // resume test in docs/ARCHITECTURE.md exists to catch — treat a
    // failure there as a No-Go signal, not a bug to silently patch
    // here.
    let _ = iced_winit::ANDROID_APP.set(app);

    if let Err(error) = crate::run() {
        // No UI exists yet to report this through; `adb logcat` is
        // the only avenue at this point in the lifecycle.
        eprintln!("app-ui exited with an error: {error}");
    }
}
