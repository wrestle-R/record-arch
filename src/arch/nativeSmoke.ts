import { rpc } from "@/desktop/transport";
import { NativeExporter } from "./NativeExporter";
/** Opt-in development diagnostic, activated by the native test harness only. */
export async function runNativeSmoke() {
	try {
		const source = await rpc("get-current-video-path");
		const resource = await rpc("get-local-media-url", source.path);
		const video = document.createElement("video");
		video.crossOrigin = "anonymous";
		video.muted = true;
		video.src = resource.url;
		await new Promise<void>((resolve, reject) => {
			video.onloadeddata = () => resolve();
			video.onerror = () => reject(new Error(`WebKit playback: ${video.error?.message}`));
			setTimeout(() => reject(new Error("WebKit playback timed out")), 15000);
		});
		const exporter = new NativeExporter({
			videoUrl: resource.url,
			width: 640,
			height: 360,
			frameRate: 15,
			bitrate: 2_000_000,
			wallpaper: "#1b2a33",
			zoomRegions: [],
			showShadow: false,
			shadowIntensity: 0,
			backgroundBlur: 0,
			borderRadius: 8,
			padding: 10,
			cropRegion: { x: 0, y: 0, width: 1, height: 1 },
			showCursor: false,
			annotationRegions: [],
			clipRegions: [{ id: "smoke", startMs: 0, endMs: 1000, sourceStartMs: 0, speed: 1 }],
		});
		const result = await exporter.export();
		await rpc("arch-smoke-report", {
			...result,
			playbackWidth: video.videoWidth,
			playbackHeight: video.videoHeight,
			userAgent: navigator.userAgent,
		});
	} catch (error) {
		await rpc("arch-smoke-report", { success: false, error: String(error) });
	}
}
