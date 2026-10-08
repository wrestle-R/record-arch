let active = false;

/** Reserve the effects renderer and let preview playback yield its GPU work. */
export function beginNativeRender() {
	if (active) throw new Error("Another export is already rendering.");
	active = true;
	window.dispatchEvent(new CustomEvent("arch-native-rendering", { detail: true }));
	return () => {
		active = false;
		window.dispatchEvent(new CustomEvent("arch-native-rendering", { detail: false }));
	};
}
