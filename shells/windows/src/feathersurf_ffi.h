// FeatherSurf FFI Bridge Header
//
// C header for calling into the Rust FFI functions.
// Auto-generated from rust/ffi/src/lib.rs

#pragma once

#include <stdint.h>
#include <stdbool.h>

#ifdef _WIN32
    #ifdef FEATHERSURF_FFI_EXPORTS
        #define FFI_EXPORT extern "C" __declspec(dllexport)
    #else
        #define FFI_EXPORT extern "C" __declspec(dllimport)
    #endif
#else
    #define FFI_EXPORT extern "C"
#endif

// Opaque handles
struct FfiTab;
struct FfiEvictionManager;

// ── Tab Lifecycle ─────────────────────────────────────────────────

// Create a new tab. Returns a handle or null on failure.
FFI_EXPORT struct FfiTab* feathersurf_tab_create(uint64_t id, const char* url);

// Destroy a tab and free its memory.
FFI_EXPORT void feathersurf_tab_destroy(struct FfiTab* tab);

// Get the current state of a tab. Returns 0xFF on null.
//   0=Active, 1=RecentlyActive, 2=Background, 3=Frozen, 4=Suspended, 5=Discardable
FFI_EXPORT uint32_t feathersurf_tab_state(const struct FfiTab* tab);

// Transition a tab to a new state. Returns 0 on success.
FFI_EXPORT uint32_t feathersurf_tab_transition(struct FfiTab* tab, uint32_t target_state);

// Set protection flags on a tab.
FFI_EXPORT void feathersurf_tab_set_protection(struct FfiTab* tab, bool pinned,
    bool media_playing, bool dirty_form, bool keep_awake);

// Force-suspend a tab, bypassing all protection flags. Returns 0 on success.
FFI_EXPORT uint32_t feathersurf_tab_force_suspend(struct FfiTab* tab);

// Get the tab's current URL. Caller must free with feathersurf_string_free.
FFI_EXPORT const char* feathersurf_tab_url(const struct FfiTab* tab);

// Get the tab's current title. Caller must free with feathersurf_string_free.
FFI_EXPORT char* feathersurf_tab_title(const struct FfiTab* tab);

// Free a string returned by FFI functions.
FFI_EXPORT void feathersurf_string_free(char* s);

// ── Eviction Manager ─────────────────────────────────────────────

// Create a new eviction manager. mode: 0=Lite, 1=Balanced, 2=Performance.
FFI_EXPORT struct FfiEvictionManager* feathersurf_eviction_create(uint32_t mode);

// Destroy an eviction manager.
FFI_EXPORT void feathersurf_eviction_destroy(struct FfiEvictionManager* mgr);

// Add a tab to the eviction manager. now = current timestamp in seconds.
FFI_EXPORT uint32_t feathersurf_eviction_add_tab(struct FfiEvictionManager* mgr,
    struct FfiTab* tab, uint64_t now);

// Update observations for a tab.
FFI_EXPORT uint32_t feathersurf_eviction_update_observation(
    struct FfiEvictionManager* mgr, uint64_t tab_id,
    float activity, float media, float user_interaction, float network,
    float memory, float cpu, float pinned, float foreground);

// Run a tick of the eviction engine. Returns number of eviction events.
FFI_EXPORT uint32_t feathersurf_eviction_tick(struct FfiEvictionManager* mgr, uint64_t now);
