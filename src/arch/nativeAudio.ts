import type { VideoExporter } from "@/lib/exporter/videoExporter";
import { getLocalFilePath } from "@/lib/exporter/localMediaSource";
import { getSourceTrackIdFromPath } from "@/lib/exporter/audioRoutingEngine";
import { resolveSourceTrackRoutingPolicy } from "@/lib/exporter/sourceTrackRoutingPolicy";
import type { Segment } from "./nativeTimeline";

type Config = Pick<
	ConstructorParameters<typeof VideoExporter>[0],
	| "sourceAudioFallbackPaths"
	| "sourceAudioTrackSettings"
	| "sourceAudioFallbackStartDelayMsByPath"
	| "audioRegions"
>;
export type NativeAudioTrack = {
	path: string;
	sourceStart: number;
	sourceEnd: number;
	outputStart: number;
	speed: number;
	volume: number;
	normalize: boolean;
};

/** Use the editor's routing policy to avoid mixing the embedded audio twice. */
export function buildNativeAudioTracks(path: string, segments: Segment[], config: Config) {
	const policy = resolveSourceTrackRoutingPolicy(path, config.sourceAudioFallbackPaths);
	const paths = [...(policy.includeEmbeddedInExport ? [path] : []), ...policy.playbackPaths];
	const tracks: NativeAudioTrack[] = paths.flatMap((resource) => {
		const audioPath = getLocalFilePath(resource) ?? resource;
		const id =
			audioPath === path
				? policy.pathsByTrack.mic
					? "system"
					: "mixed"
				: getSourceTrackIdFromPath(audioPath);
		const setting = config.sourceAudioTrackSettings?.[id];
		const volume = setting?.volume ?? 1;
		const delay =
			Math.max(0, config.sourceAudioFallbackStartDelayMsByPath?.[resource] ?? 0) / 1000;
		return segments
			.filter((s) => !s.muted && volume > 0 && s.endSec > delay)
			.map((s) => {
				const start = Math.max(s.startSec, delay);
				return {
					path: audioPath,
					sourceStart: start - delay,
					sourceEnd: s.endSec - delay,
					outputStart: s.outputStart + (start - s.startSec) / s.speed,
					speed: s.speed,
					volume,
					normalize: setting?.normalize ?? false,
				};
			});
	});

	for (const audio of config.audioRegions ?? []) {
		tracks.push({
			path: getLocalFilePath(audio.audioPath) ?? audio.audioPath,
			sourceStart: 0,
			sourceEnd: (audio.endMs - audio.startMs) / 1000,
			outputStart: audio.startMs / 1000,
			speed: 1,
			volume: audio.volume,
			normalize: audio.normalize ?? false,
		});
	}
	return tracks;
}
