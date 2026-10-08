import type { GifExporter } from "@/lib/exporter/gifExporter";
import type { ExportResult } from "@/lib/exporter/types";
import { rpc } from "@/desktop/transport";
import { NativeExporter } from "./NativeExporter";
type Config = ConstructorParameters<typeof GifExporter>[0];
export class NativeGifExporter {
	private encoder: NativeExporter;
	constructor(private config: Config) {
		const { videoPadding, ...render } = config;
		this.encoder = new NativeExporter({
			...render,
			padding: videoPadding ?? config.padding,
			bitrate: 4_000_000,
		});
	}
	cancel() {
		this.encoder.cancel();
	}
	async export(): Promise<ExportResult> {
		const video = await this.encoder.export();
		if (!video.success || !video.tempFilePath) return video;
		let gif: string | undefined;
		try {
			const result = await rpc("arch-convert-gif", video.tempFilePath, {
				loop: this.config.loop,
			});
			if (!result.success) throw new Error(result.error);
			gif = result.path;
			const resource = await rpc("get-local-media-url", gif);
			const response = await fetch(resource.url);
			if (!response.ok) throw new Error("Could not read the exported GIF");
			return { success: true, blob: await response.blob() };
		} catch (error) {
			return { success: false, error: String(error) };
		} finally {
			await rpc("discard-exported-temp", video.tempFilePath).catch(() => {});
			if (gif) await rpc("discard-exported-temp", gif).catch(() => {});
		}
	}
}
