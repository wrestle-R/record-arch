import { describe, expect, it } from "vitest";
import { buildNativeAudioTracks } from "./nativeAudio";
import type { Segment } from "./nativeTimeline";

const segment: Segment = { startSec: 0, endSec: 2, outputStart: 0, outputEnd: 2, speed: 1 };

describe("native companion audio routing", () => {
	it("uses dedicated tracks without doubling the embedded mix", () => {
		const tracks = buildNativeAudioTracks("/video.mp4", [segment], {
			sourceAudioFallbackPaths: ["/video.system.wav", "/video.microphone.wav"],
		});
		expect(tracks.map((t) => t.path)).toEqual(["/video.system.wav", "/video.microphone.wav"]);
	});
	it("honors microphone and system muting when only the mic is separate", () => {
		const tracks = buildNativeAudioTracks("/video.mp4", [segment], {
			sourceAudioFallbackPaths: ["/video.microphone.wav"],
			sourceAudioTrackSettings: {
				mic: { volume: 0, normalize: false },
				system: { volume: 0, normalize: false },
			},
		});
		expect(tracks).toEqual([]);
	});
	it("aligns late audio with source offsets and speed changes", () => {
		const tracks = buildNativeAudioTracks(
			"/video.mp4",
			[{ startSec: 1, endSec: 5, outputStart: 3, outputEnd: 5, speed: 2 }],
			{
				sourceAudioFallbackPaths: ["/video.mic.wav"],
				sourceAudioFallbackStartDelayMsByPath: { "/video.mic.wav": 2000 },
			},
		);
		expect(tracks.find((t) => t.path.endsWith(".mic.wav"))).toMatchObject({
			sourceStart: 0,
			sourceEnd: 3,
			outputStart: 3.5,
			speed: 2,
		});
	});
});
