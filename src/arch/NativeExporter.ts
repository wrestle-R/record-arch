import { FrameRenderer } from "@/lib/exporter/frameRenderer";
import { getLocalFilePath } from "@/lib/exporter/localMediaSource";
import type { VideoExporter } from "@/lib/exporter/videoExporter";
import type { ExportResult } from "@/lib/exporter/types";
import { rpc } from "@/desktop/transport";
import { buildNativeTimeline } from "./nativeTimeline";
import { beginNativeRender } from "./nativeRenderSession";
import { buildNativeAudioTracks } from "./nativeAudio";
type Config = ConstructorParameters<typeof VideoExporter>[0];
export class NativeExporter {
	private cancelled = false;
	private session: string | null = null;
	private decoder: string | null = null;
	constructor(private config: Config) {}
	cancel() {
		this.cancelled = true;
		if (this.session) void rpc("arch-encode-cancel", this.session);
	}
	async export(): Promise<ExportResult> {
		const config = this.config;
		let renderer: FrameRenderer | null = null;
		let audioPath: string | null = null;
		let releaseRender: (() => void) | undefined;
		try {
			releaseRender = beginNativeRender();
			await new Promise<void>((resolve) => setTimeout(resolve, 0));
			const path = getLocalFilePath(config.videoUrl);
			if (!path) throw new Error("Native export requires a local video.");
			const probe = await rpc("probe-native-video-metadata", path);
			if (!probe.success) throw new Error(probe.error);
			const metadata = probe.metadata;
			const segments = buildNativeTimeline(
				metadata.duration,
				config.clipRegions,
				config.trimRegions,
				config.speedRegions,
			);
			const duration = Math.max(0, ...segments.map((s) => s.outputEnd));
			if (!duration) throw new Error("The timeline is empty.");
			renderer = new FrameRenderer({
				...config,
				videoWidth: metadata.width,
				videoHeight: metadata.height,
				preferredRenderBackend: "webgl",
			});
			await renderer.initialize();
			const started = await rpc("arch-encode-start", config);
			if (!started.success) throw new Error(started.error);
			this.session = started.sessionId;
			const total = Math.ceil(duration * config.frameRate);
			const start = performance.now();
			let activeSegment: (typeof segments)[number] | undefined;
			const canvas = document.createElement("canvas");
			canvas.width = metadata.width;
			canvas.height = metadata.height;
			const context = canvas.getContext("2d")!;
			for (let index = 0; index < total; index++) {
				if (this.cancelled) throw new Error("Export cancelled");
				const time = index / config.frameRate;
				const segment = segments.find((s) => time >= s.outputStart && time < s.outputEnd);
				const sourceTime = segment
					? Math.min(
							segment.endSec - 1 / (config.frameRate * 2),
							segment.startSec + (time - segment.outputStart) * segment.speed,
						)
					: 0;
				if (segment) {
					if (activeSegment !== segment) {
						if (this.decoder) await rpc("arch-decoder-close", this.decoder);
						const decoded = await rpc(
							"arch-decoder-open",
							path,
							sourceTime,
							config.frameRate / segment.speed,
						);
						if (!decoded.success) throw new Error(decoded.error);
						this.decoder = decoded.sessionId;
						activeSegment = segment;
					}
					const frame = await rpc("arch-decoder-next", this.decoder);
					if (!frame.success) throw new Error(frame.error);
					const image = new Image();
					image.src = frame.dataUrl;
					await image.decode();
					context.drawImage(image, 0, 0);
					// Pixi accepts CanvasImageSource. The editor renderer does not depend on VideoFrame metadata.
					await renderer.renderFrame(
						canvas as unknown as VideoFrame,
						time * 1e6,
						sourceTime * 1e6,
						1e6 / config.frameRate,
						time * 1e6,
					);
				} else
					await renderer.renderFrame(
						null,
						time * 1e6,
						time * 1e6,
						1e6 / config.frameRate,
						time * 1e6,
					);
				const blob = await new Promise<Blob>((resolve, reject) =>
					renderer!
						.getCanvas()
						.toBlob(
							(b) => (b ? resolve(b) : reject(new Error("Frame capture failed"))),
							"image/png",
						),
				);
				const result = await rpc(
					"arch-encode-frame",
					this.session,
					await blob.arrayBuffer(),
				);
				if (!result.success) throw new Error(result.error);
				const elapsed = (performance.now() - start) / 1000;
				config.onProgress?.({
					currentFrame: index + 1,
					totalFrames: total,
					percentage: ((index + 1) / total) * 95,
					estimatedTimeRemaining: (elapsed / (index + 1)) * (total - index - 1),
					renderBackend: "webgl",
					encodeBackend: "ffmpeg",
					encoderName: "libx264",
				});
			}
			const tracks = buildNativeAudioTracks(path, segments, config);
			const mixed = await rpc("arch-mix-audio", { duration, tracks });
			if (!mixed.success) throw new Error(mixed.error);
			audioPath = mixed.path;
			const result = await rpc("arch-encode-finish", this.session, { audioPath: mixed.path });
			if (!result.success) throw new Error(result.error);
			this.session = null;
			return { success: true, tempFilePath: result.tempPath };
		} catch (error) {
			return { success: false, error: String(error) };
		} finally {
			renderer?.destroy();
			if (audioPath) await rpc("discard-exported-temp", audioPath).catch(() => {});
			if (this.decoder) {
				await rpc("arch-decoder-close", this.decoder).catch(() => {});
				this.decoder = null;
			}
			if (this.session) {
				await rpc("arch-encode-cancel", this.session).catch(() => {});
				this.session = null;
			}
			releaseRender?.();
		}
	}
}
