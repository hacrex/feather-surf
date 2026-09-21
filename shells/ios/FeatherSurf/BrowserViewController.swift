import UIKit
import WebKit

/// Main view controller for FeatherSurf iOS browser.
///
/// Uses WKWebView (WebKit) for rendering.
/// Apple mandates WebKit on iOS, so Chromium/CEF cannot be used.
/// The Rust core manages tab lifecycle and memory policy identically
/// to desktop platforms.
class BrowserViewController: UIViewController {

    // MARK: - Properties

    private var webView: WKWebView!
    private var urlBar: UITextField!
    private var tabId: UInt64 = 0

    // MARK: - Lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()
        setupUI()
        setupRustCore()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        // Notify Rust core: tab is active
        _ = FeatherSurfNative.tabTransition(UnsafeRawPointer(bitPattern: tabId)!, to: .active)
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        // Notify Rust core: tab is going to background
        _ = FeatherSurfNative.tabTransition(UnsafeRawPointer(bitPattern: tabId)!, to: .background)
    }

    // MARK: - Setup

    private func setupUI() {
        view.backgroundColor = .systemBackground

        // URL bar
        urlBar = UITextField()
        urlBar.borderStyle = .roundedRect
        urlBar.placeholder = "Enter URL"
        urlBar.keyboardType = .URL
        urlBar.autocapitalizationType = .none
        urlBar.autocorrectionType = .no
        urlBar.returnKeyType = .go
        urlBar.delegate = self
        urlBar.translatesAutoresizingMaskIntoConstraints = false

        let goButton = UIButton(type: .system)
        goButton.setTitle("Go", for: .normal)
        goButton.addTarget(self, action: #selector(navigateToUrl), for: .touchUpInside)
        goButton.translatesAutoresizingMaskIntoConstraints = false

        // WebView
        let config = WKWebViewConfiguration()
        config.allowsInlineMediaPlayback = true
        webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = self
        webView.translatesAutoresizingMaskIntoConstraints = false

        // Layout
        view.addSubview(urlBar)
        view.addSubview(goButton)
        view.addSubview(webView)

        NSLayoutConstraint.activate([
            urlBar.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 8),
            urlBar.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 8),
            urlBar.trailingAnchor.constraint(equalTo: goButton.leadingAnchor, constant: -8),
            urlBar.heightAnchor.constraint(equalToConstant: 40),

            goButton.centerYAnchor.constraint(equalTo: urlBar.centerYAnchor),
            goButton.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -8),
            goButton.widthAnchor.constraint(equalToConstant: 50),

            webView.topAnchor.constraint(equalTo: urlBar.bottomAnchor, constant: 8),
            webView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            webView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            webView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func setupRustCore() {
        // Create a tab in the Rust core
        // Note: The actual FFI call would use a C bridging header
        // For now, this is a placeholder showing the intended flow
        if let url = URL(string: "https://www.google.com") {
            webView.load(URLRequest(url: url))
        }
    }

    // MARK: - Actions

    @objc private func navigateToUrl() {
        guard let text = urlBar.text, !text.isEmpty else { return }

        var urlString = text
        if !text.hasPrefix("http://") && !text.hasPrefix("https://") {
            urlString = "https://\(text)"
        }

        if let url = URL(string: urlString) {
            webView.load(URLRequest(url: url))
        }
    }
}

// MARK: - UITextFieldDelegate

extension BrowserViewController: UITextFieldDelegate {
    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        textField.resignFirstResponder()
        navigateToUrl()
        return true
    }
}

// MARK: - WKNavigationDelegate

extension BrowserViewController: WKNavigationDelegate {
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        urlBar.text = webView.url?.absoluteString
        title = webView.title
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        print("Navigation failed: \(error.localizedDescription)")
    }
}
