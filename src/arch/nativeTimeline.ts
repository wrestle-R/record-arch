import {
	getClipSourceStartMs,
	type ClipRegion,
	type SpeedRegion,
	type TrimRegion,
} from "@/components/video-editor/types";
import {
	computeVideoSegments,
	splitVideoSegmentsBySpeed,
} from "@/lib/exporter/videoTimelineSegments";
export type Segment = {
	startSec: number;
	endSec: number;
	speed: number;
	outputStart: number;
	outputEnd: number;
	muted?: boolean;
};
export function buildNativeTimeline(
	duration: number,
	clips?: ClipRegion[],
	trims?: TrimRegion[],
	speeds?: SpeedRegion[],
): Segment[] {
	if (clips?.length)
		return [...clips]
			.sort((a, b) => a.startMs - b.startMs)
			.map((c) => ({
				startSec: getClipSourceStartMs(c) / 1000,
				endSec: (getClipSourceStartMs(c) + (c.endMs - c.startMs) * c.speed) / 1000,
				speed: c.speed,
				outputStart: c.startMs / 1000,
				outputEnd: c.endMs / 1000,
				muted: c.muted,
			}));
	let offset = 0;
	return splitVideoSegmentsBySpeed(computeVideoSegments(duration, trims), speeds).map((s) => {
		const outputStart = offset;
		offset += (s.endSec - s.startSec) / s.speed;
		return { ...s, outputStart, outputEnd: offset };
	});
}
