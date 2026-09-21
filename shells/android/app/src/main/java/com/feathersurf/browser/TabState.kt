package com.feathersurf.browser

/**
 * Tab state constants matching the Rust TabState enum.
 */
object TabState {
    const val ACTIVE = 0u
    const val RECENTLY_ACTIVE = 1u
    const val BACKGROUND = 2u
    const val FROZEN = 3u
    const val SUSPENDED = 4u
    const val DISCARDABLE = 5u
}
