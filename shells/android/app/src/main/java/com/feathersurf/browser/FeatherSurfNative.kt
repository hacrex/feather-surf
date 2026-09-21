package com.feathersurf.browser

/**
 * JNI bridge to the FeatherSurf Rust core.
 *
 * The native library (libfeathersurf_ffi.so) is loaded at class init time.
 * All methods are thin wrappers around the C FFI functions defined in ffi/src/lib.rs.
 */
object FeatherSurfNative {

    init {
        System.loadLibrary("feathersurf_ffi")
    }

    // -- Native methods (implemented in Rust via JNI) --

    /**
     * Initialize the Rust core. Must be called once before any other method.
     */
    external fun nativeInit()

    /**
     * Create a new tab with the given URL.
     * Returns the tab ID (u64).
     */
    external fun nativeTabCreate(url: String): Long

    /**
     * Destroy a tab and free its resources.
     */
    external fun nativeTabDestroy(tabId: Long)

    /**
     * Get the current state of a tab.
     * Returns a TabState constant.
     */
    external fun nativeTabState(tabId: Long): Int

    /**
     * Transition a tab to a new state.
     * Returns 0 on success, non-zero on failure.
     */
    external fun nativeTabTransition(tabId: Long, targetState: Int): Int

    /**
     * Get the current URL of a tab.
     */
    external fun nativeTabUrl(tabId: Long): String

    /**
     * Get the current title of a tab.
     */
    external fun nativeTabTitle(tabId: Long): String

    // -- Kotlin-friendly wrappers --

    fun init() {
        nativeInit()
    }

    fun tabCreate(url: String): Long {
        return nativeTabCreate(url)
    }

    fun tabDestroy(tabId: Long) {
        nativeTabDestroy(tabId)
    }

    fun tabState(tabId: Long): Int {
        return nativeTabState(tabId)
    }

    fun tabTransition(tabId: Long, targetState: Int): Boolean {
        return nativeTabTransition(tabId, targetState) == 0
    }

    fun tabUrl(tabId: Long): String {
        return nativeTabUrl(tabId)
    }

    fun tabTitle(tabId: Long): String {
        return nativeTabTitle(tabId)
    }
}
