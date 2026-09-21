package com.feathersurf.browser

import android.app.Activity
import android.os.Bundle
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.LinearLayout
import android.widget.EditText
import android.widget.Button

/**
 * Main Activity for FeatherSurf Android browser.
 *
 * Uses Android WebView for rendering (Chromium-based).
 * Communicates with the Rust core via JNI through FeatherSurfNative.
 */
class BrowserActivity : Activity() {

    private lateinit var webView: WebView
    private lateinit var urlBar: EditText
    private lateinit var goButton: Button
    private var tabId: Long = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Initialize the Rust core
        FeatherSurfNative.init()

        // Create a tab in the Rust core
        tabId = FeatherSurfNative.tabCreate("https://www.google.com")

        // Build the UI
        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
        }

        // URL bar
        val urlBarLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
        }

        urlBar = EditText(this).apply {
            hint = "Enter URL"
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
        }

        goButton = Button(this).apply {
            text = "Go"
            setOnClickListener { navigateToUrl() }
        }

        urlBarLayout.addView(urlBar)
        urlBarLayout.addView(goButton)

        // WebView
        webView = WebView(this).apply {
            webViewClient = object : WebViewClient() {
                override fun onPageFinished(view: WebView?, url: String?) {
                    urlBar.setText(url)
                    // Update Rust core with new URL
                    url?.let { FeatherSurfNative.tabNavigate(tabId, it) }
                }
            }
            settings.javaScriptEnabled = true
            settings.domStorageEnabled = true
        }

        layout.addView(urlBarLayout)
        layout.addView(webView)

        setContentView(layout)

        // Load initial URL
        webView.loadUrl("https://www.google.com")
    }

    private fun navigateToUrl() {
        val url = urlBar.text.toString().trim()
        if (url.isNotEmpty()) {
            val fullUrl = if (!url.startsWith("http://") && !url.startsWith("https://")) {
                "https://$url"
            } else {
                url
            }
            webView.loadUrl(fullUrl)
        }
    }

    override fun onPause() {
        super.onPause()
        webView.onPause()
        // Notify Rust core that the tab is going to background
        FeatherSurfNative.tabTransition(tabId, TabState.BACKGROUND)
    }

    override fun onResume() {
        super.onResume()
        webView.onResume()
        // Notify Rust core that the tab is active
        FeatherSurfNative.tabTransition(tabId, TabState.ACTIVE)
    }

    override fun onDestroy() {
        webView.destroy()
        FeatherSurfNative.tabDestroy(tabId)
        super.onDestroy()
    }
}
