//! C FFI bridge for FeatherSurf.
//!
//! Exposes a stable C ABI that can be called from:
//! - Android via JNI (Kotlin/Java)
//! - iOS via Swift bridging header
//! - Desktop via CEF C API integration
//!
//! All functions use opaque handles to avoid exposing Rust internals.
//! Error handling uses error codes with an optional message buffer.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use memory_manager::{EvictionManager, MemoryMode};
use tab_manager::{Tab, TabProtection, TabState};

// ---------------------------------------------------------------------------
// Opaque handles
// ---------------------------------------------------------------------------

/// Opaque handle to a Tab instance.
pub struct FfiTab {
    pub inner: Tab,
}

/// Opaque handle to an EvictionManager instance.
pub struct FfiEvictionManager {
    pub inner: Mutex<EvictionManager>,
}

// ---------------------------------------------------------------------------
// Tab lifecycle
// ---------------------------------------------------------------------------

/// Create a new tab. Returns a handle or null on failure.
///
/// # Safety
/// `url` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_create(id: u64, url: *const c_char) -> *mut FfiTab {
    let url_str = match unsafe { CStr::from_ptr(url) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let tab = Tab::new(id, url_str);
    Box::into_raw(Box::new(FfiTab { inner: tab }))
}

/// Destroy a tab and free its memory.
///
/// # Safety
/// `tab` must be a valid pointer returned by `feathersurf_tab_create`.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_destroy(tab: *mut FfiTab) {
    if !tab.is_null() {
        unsafe { drop(Box::from_raw(tab)) };
    }
}

/// Get the current state of a tab.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_state(tab: *const FfiTab) -> u32 {
    if tab.is_null() {
        return 0xFF;
    }
    let tab = unsafe { &*tab };
    match tab.inner.state() {
        TabState::Active => 0,
        TabState::RecentlyActive => 1,
        TabState::Background => 2,
        TabState::Frozen => 3,
        TabState::Suspended => 4,
        TabState::Discardable => 5,
    }
}

/// Transition a tab to a new state.
/// Returns 0 on success, non-zero error code on failure.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_transition(tab: *mut FfiTab, target_state: u32) -> u32 {
    if tab.is_null() {
        return 1;
    }
    let tab = unsafe { &mut *tab };
    let target = match target_state {
        0 => TabState::Active,
        1 => TabState::RecentlyActive,
        2 => TabState::Background,
        3 => TabState::Frozen,
        4 => TabState::Suspended,
        5 => TabState::Discardable,
        _ => return 2,
    };
    match tab.inner.transition_to(target) {
        Ok(()) => 0,
        Err(_) => 3,
    }
}

/// Set protection flags on a tab.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_set_protection(
    tab: *mut FfiTab,
    pinned: bool,
    media_playing: bool,
    dirty_form: bool,
    keep_awake: bool,
) {
    if tab.is_null() {
        return;
    }
    let tab = unsafe { &mut *tab };
    tab.inner.set_protection(TabProtection {
        pinned,
        media_playing,
        dirty_form,
        keep_awake,
    });
}

/// Force-suspend a tab, bypassing all protection flags.
/// Returns 0 on success.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_force_suspend(tab: *mut FfiTab) -> u32 {
    if tab.is_null() {
        return 1;
    }
    let tab = unsafe { &mut *tab };
    match tab.inner.force_suspend() {
        Ok(_) => 0,
        Err(_) => 2,
    }
}

/// Get the tab's current URL. Returns a pointer to a static string.
///
/// # Safety
/// The returned pointer is valid until the tab is destroyed.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_url(tab: *const FfiTab) -> *const c_char {
    if tab.is_null() {
        return std::ptr::null();
    }
    let tab = unsafe { &*tab };
    match CString::new(tab.inner.snapshot().url.clone()) {
        Ok(s) => s.into_raw() as *const c_char,
        Err(_) => std::ptr::null(),
    }
}

/// Get the tab's current title.
///
/// # Safety
/// Caller must free the returned string with `feathersurf_string_free`.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_tab_title(tab: *const FfiTab) -> *mut c_char {
    if tab.is_null() {
        return std::ptr::null_mut();
    }
    let tab = unsafe { &*tab };
    match CString::new(tab.inner.snapshot().title.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by FFI functions.
///
/// # Safety
/// `s` must be a pointer returned by an FFI function.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ---------------------------------------------------------------------------
// Eviction manager
// ---------------------------------------------------------------------------

/// Create a new eviction manager.
#[no_mangle]
pub extern "C" fn feathersurf_eviction_create(mode: u32) -> *mut FfiEvictionManager {
    let memory_mode = match mode {
        0 => MemoryMode::Lite,
        1 => MemoryMode::Balanced,
        2 => MemoryMode::Performance,
        _ => MemoryMode::Balanced,
    };
    let config = memory_manager::PolicyConfig {
        mode: memory_mode,
        ..memory_manager::PolicyConfig::default()
    };
    let manager = EvictionManager::new(config, None);
    Box::into_raw(Box::new(FfiEvictionManager {
        inner: Mutex::new(manager),
    }))
}

/// Destroy an eviction manager.
///
/// # Safety
/// `mgr` must be a valid pointer returned by `feathersurf_eviction_create`.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_eviction_destroy(mgr: *mut FfiEvictionManager) {
    if !mgr.is_null() {
        unsafe { drop(Box::from_raw(mgr)) };
    }
}

/// Add a tab to the eviction manager.
/// `now` is the current timestamp in seconds (monotonic clock).
#[no_mangle]
pub unsafe extern "C" fn feathersurf_eviction_add_tab(
    mgr: *const FfiEvictionManager,
    tab: *mut FfiTab,
    now: u64,
) -> u32 {
    if mgr.is_null() || tab.is_null() {
        return 1;
    }
    let mgr = unsafe { &*mgr };
    let tab = unsafe { &mut *tab };
    let mut inner = mgr.inner.lock().unwrap();
    let obs = memory_manager::TabObservation::idle();
    inner.add_tab(tab.inner.clone(), obs, now);
    0
}

/// Update observations for a tab.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_eviction_update_observation(
    mgr: *const FfiEvictionManager,
    tab_id: u64,
    activity: f32,
    media: f32,
    user_interaction: f32,
    network: f32,
    memory: f32,
    cpu: f32,
    pinned: f32,
    foreground: f32,
) -> u32 {
    if mgr.is_null() {
        return 1;
    }
    let mgr = unsafe { &*mgr };
    let mut inner = mgr.inner.lock().unwrap();
    let obs = memory_manager::TabObservation {
        activity,
        media,
        user_interaction,
        network,
        memory,
        cpu,
        pinned,
        foreground,
    };
    inner.update_observation(tab_id, obs);
    0
}

/// Run a tick of the eviction engine.
/// `now` is the current timestamp in seconds (monotonic clock).
/// Returns the number of eviction events generated.
#[no_mangle]
pub unsafe extern "C" fn feathersurf_eviction_tick(mgr: *const FfiEvictionManager, now: u64) -> u32 {
    if mgr.is_null() {
        return 0;
    }
    let mgr = unsafe { &*mgr };
    let mut inner = mgr.inner.lock().unwrap();
    let events = inner.tick(now);
    events.len() as u32
}
