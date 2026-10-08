/** React Aria expects Screen Orientation, which desktop WebKitGTK may omit. */
export function installWebviewCompatibility() {
	if (!screen.orientation) {
		const orientation = new EventTarget();
		Object.defineProperties(orientation, {
			angle: { get: () => 0 },
			type: {
				get: () =>
					screen.width >= screen.height ? "landscape-primary" : "portrait-primary",
			},
			lock: {
				value: () =>
					Promise.reject(
						new DOMException(
							"Orientation locking is unavailable on this desktop",
							"NotSupportedError",
						),
					),
			},
			unlock: { value: () => undefined },
		});
		Object.defineProperty(screen, "orientation", { value: orientation, configurable: true });
	}
}
