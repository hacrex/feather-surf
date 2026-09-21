import UIKit

/// App delegate for FeatherSurf iOS.
@main
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        window = UIWindow(frame: UIScreen.main.bounds)
        let browserVC = BrowserViewController()
        window?.rootViewController = UINavigationController(rootViewController: browserVC)
        window?.makeKeyAndVisible()
        return true
    }
}
