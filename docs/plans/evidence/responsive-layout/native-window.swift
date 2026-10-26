import CoreGraphics
import Foundation
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
guard let window = windows.first(where: { ($0[kCGWindowName as String] as? String) == "Meetily Responsive Test" }),
      let bounds = window[kCGWindowBounds as String] as? [String: Any] else {
    fatalError("Isolated responsive native window is not on screen")
}
let json = try JSONSerialization.data(withJSONObject: bounds)
print(String(data: json, encoding: .utf8)!)
