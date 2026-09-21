// FeatherSurf FFI Bridge Header
//
// C++ header for calling into the Rust FFI functions.
// Generated from rust/ffi/src/lib.rs

#pragma once

#include <cstdint>

#ifdef _WIN32
    #define FFI_EXPORT extern "C" __declspec(dllimport)
#else
    #define FFI_EXPORT extern "C"
#endif

// Opaque handles
struct FfiTab;
struct FfiEvictionManager;

// Tab lifecycle
FFI_EXPORT FfiTab* feathersurf_tab_create(uint64_t id, const char* url);
FFI_EXPORT void feathersurf_tab_destroy(FfiTab* tab);
FFI_EXPORT uint64_t feathersurf_tab_id(const FfiTab* tab);
FFI_EXPORT uint32_t feathersurf_tab_state(const FfiTab* tab);
FFI_EXPORT const char* feathersurf_tab_url(const FfiTab* tab);
FFI_EXPORT uint32_t feathersurf_tab_revision(const FfiTab* tab);

// Tab transitions
FFI_EXPORT uint32_t feathersurf_tab_transition(FfiTab* tab, uint32_t target_state);
FFI_EXPORT uint32_t feathersurf_tab_force_suspend(FfiTab* tab);

// Tab protection
FFI_EXPORT void feathersurf_tab_set_protection(FfiTab* tab, bool pinned,
    bool media_playing, bool dirty_form, bool keep_awake);

// Tab serialization
FFI_EXPORT uint32_t feathersurf_tab_snapshot_size(const FfiTab* tab);
FFI_EXPORT uint32_t feathersurf_tab_snapshot_write(const FfiTab* tab,
    uint8_t* buf, uint32_t buf_len);
FFI_EXPORT uint32_t feathersurf_tab_snapshot_read(FfiTab* tab,
    const uint8_t* buf, uint32_t buf_len);

// Eviction manager
FFI_EXPORT FfiEvictionManager* feathersurf_eviction_create(
    float candidate_threshold, float scoring_exponent, uint32_t max_samples);
FFI_EXPORT void feathersurf_eviction_destroy(FfiEvictionManager* mgr);
FFI_EXPORT void feathersurf_eviction_add_tab(FfiEvictionManager* mgr,
    FfiTab* tab, uint64_t now);
FFI_EXPORT void feathersurf_eviction_remove_tab(FfiEvictionManager* mgr,
    uint64_t tab_id);
FFI_EXPORT uint32_t feathersurf_eviction_next_to_freeze(
    const FfiEvictionManager* mgr);
FFI_EXPORT uint32_t feathersurf_eviction_next_to_suspend(
    const FfiEvictionManager* mgr);
FFI_EXPORT float feathersurf_eviction_score(const FfiEvictionManager* mgr,
    uint64_t tab_id);
FFI_EXPORT uint32_t feathersurf_eviction_mark_frozen(FfiEvictionManager* mgr,
    uint64_t tab_id, uint64_t now);
FFI_EXPORT uint32_t feathersurf_eviction_mark_suspended(FfiEvictionManager* mgr,
    uint64_t tab_id, uint64_t now);
FFI_EXPORT uint32_t feathersurf_eviction_mark_discarded(FfiEvictionManager* mgr,
    uint64_t tab_id, uint64_t now);
FFI_EXPORT uint32_t feathersurf_eviction_mark_active(FfiEvictionManager* mgr,
    uint64_t tab_id, uint64_t now);
