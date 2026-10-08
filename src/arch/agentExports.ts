import { ipcRenderer, rpc } from "@/desktop/transport";
import {
	normalizeProjectEditor,
	type ProjectEditorState,
} from "@/components/video-editor/projectPersistence";
import { NativeExporter } from "./NativeExporter";
import { NativeGifExporter } from "./NativeGifExporter";
type Job = {
	id: string;
	project: { videoPath: string; editor: Partial<ProjectEditorState> };
	options: {
		outputPath: string;
		width?: number;
		height?: number;
		frameRate?: number;
		format?: string;
	};
};
export function installAgentExports() {
	let active: { id: string; exporter: NativeExporter | NativeGifExporter } | null = null;
	const cancelled = (_: unknown, id: string) => {
		if (active?.id === id) active.exporter.cancel();
	};
	const run = async (_: unknown, job: Job) => {
		try {
			const status = await rpc("arch-export-status", job.id);
			if (status.job?.status === "cancelled") return;
			const editor = normalizeProjectEditor(job.project.editor);
			const resource = await rpc("get-local-media-url", job.project.videoPath);
			const meta = await rpc("probe-native-video-metadata", job.project.videoPath);
			if (!meta.success) throw new Error(meta.error);
			const telemetry = await rpc("get-cursor-telemetry", job.project.videoPath);
			const sidecars = await rpc("get-video-audio-fallback-paths", job.project.videoPath);
			const webcam = editor.webcam.sourcePath
				? await rpc("get-local-media-url", editor.webcam.sourcePath)
				: null;
			const config = {
				...editor,
				videoUrl: resource.url,
				webcamUrl: webcam?.url,
				sourceAudioFallbackPaths: sidecars.paths ?? [],
				width: job.options.width ?? meta.metadata.width,
				height: job.options.height ?? meta.metadata.height,
				frameRate: job.options.frameRate ?? 30,
				bitrate: 8_000_000,
				showShadow: editor.shadowIntensity > 0,
				cursorTelemetry: telemetry.samples ?? [],
				previewWidth: meta.metadata.width,
				previewHeight: meta.metadata.height,
				onProgress: (p: { percentage: number }) => {
					void rpc("arch-export-progress", job.id, p.percentage);
				},
			};
			const exporter =
				job.options.format === "gif"
					? new NativeGifExporter({
							...config,
							frameRate: 15,
							loop: true,
							sizePreset: "original",
						})
					: new NativeExporter(config);
			active = { id: job.id, exporter };
			const result = await exporter.export();
			if (result.success && result.blob) {
				const opened = await rpc("export-stream-open", { extension: "gif" });
				await rpc(
					"export-stream-write",
					opened.streamId,
					0,
					await result.blob.arrayBuffer(),
				);
				const closed = await rpc("export-stream-close", opened.streamId);
				result.tempFilePath = closed.tempFilePath;
			}
			const response = await rpc("arch-export-complete", job.id, result);
			if (!response.success && result.tempFilePath)
				await rpc("discard-exported-temp", result.tempFilePath);
		} catch (error) {
			await rpc("arch-export-complete", job.id, {
				success: false,
				error: String(error),
			}).catch(console.error);
		} finally {
			active = null;
		}
	};
	ipcRenderer.on("arch-export-job", run);
	ipcRenderer.on("arch-export-cancelled", cancelled);
	void rpc("arch-renderer-ready");
	return () => {
		ipcRenderer.removeListener("arch-export-job", run);
		ipcRenderer.removeListener("arch-export-cancelled", cancelled);
		active?.exporter.cancel();
	};
}
