import { installWebviewCompatibility } from "./desktop/webviewCompat";
installWebviewCompatibility();
// Compatibility must run before UI libraries evaluate their browser feature probes.
void import("pixi.js/unsafe-eval")
	.then(() => import("./main"))
	.catch((error) => {
		console.error("Record Arch startup failed", error);
		const root = document.getElementById("root");
		if (root) {
			root.textContent = `Record Arch could not start: ${error instanceof Error ? error.message : String(error)}`;
			root.setAttribute("role", "alert");
		}
	});
