import { chromium } from "@playwright/test";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, writeFile, access } from "node:fs/promises";
import assert from "node:assert/strict";
import path from "node:path";
const exec = promisify(execFile);
const root = process.env.RECORD_ARCH_HOME ?? "/tmp/record-arch-integration";
const env = {
	...process.env,
	RECORD_ARCH_HOME: root,
	RECORD_ARCH_SOCKET: process.env.RECORD_ARCH_SOCKET ?? "/tmp/record-arch-integration.sock",
};
const binary = path.resolve("src-tauri/target/debug/record-arch");
async function cli(...args) {
	const { stdout } = await exec(binary, ["--json", ...args], { env, timeout: 180000 });
	return JSON.parse(stdout);
}
const browser = await chromium.launch({
	headless: true,
	args: [
		"--no-sandbox",
		"--use-gl=angle",
		"--use-angle=swiftshader",
		"--enable-unsafe-swiftshader",
	],
});
const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
const errors = [];
page.on("pageerror", (error) => errors.push(error.message));
try {
	await page.goto("http://localhost:1420");
	await page.waitForFunction(() =>
		[...document.querySelectorAll("video")].some(
			(v) => v.videoWidth === 640 && v.duration > 1.9,
		),
	);
	await page.getByText("No Video Loaded", { exact: true }).waitFor({ state: "hidden" });
	await page.keyboard.press("Control+s");
	const listed = await cli("project", "list");
	assert(listed.entries.length > 0);
	const project = listed.entries[0].path;
	const persisted = JSON.parse(await readFile(project, "utf8"));
	assert.equal(persisted.version, 2);
	assert(persisted.projectId);
	console.log("PASS: editor playback, timeline hydration and save");
	await page.getByRole("button", { name: "Export", exact: true }).click();
	console.log("EXPORT CONTROLS", (await page.locator("body").innerText()).slice(-1400));
	await page.keyboard.press("Escape");
	const output = path.join(root, "exports", `agent-test-${Date.now()}.mp4`);
	const job = await cli(
		"export",
		"start",
		project,
		"--output",
		output,
		"--width",
		"640",
		"--height",
		"360",
		"--fps",
		"15",
	);
	assert(job.job.id);
	let result;
	for (let i = 0; i < 240; i++) {
		result = await cli("export", "status", job.job.id);
		if (["completed", "failed", "cancelled"].includes(result.job.status)) break;
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	assert.equal(result.job.status, "completed", JSON.stringify(result));
	const metadata = JSON.parse(
		(
			await exec("ffprobe", [
				"-v",
				"error",
				"-show_streams",
				"-show_format",
				"-of",
				"json",
				output,
			])
		).stdout,
	);
	assert.equal(metadata.streams[0].codec_name, "h264");
	assert.equal(metadata.streams[0].width, 640);
	assert(metadata.streams.some((s) => s.codec_name === "aac"));
	assert(Math.abs(Number(metadata.format.duration) - 2) < 0.15);
	console.log("PASS: CLI submit/progress/completion, native MP4/audio, duration and dimensions");
	const cancelled = await cli(
		"export",
		"start",
		project,
		"--output",
		path.join(root, "exports", `cancelled-${Date.now()}.mp4`),
		"--fps",
		"120",
	);
	let running;
	for (let i = 0; i < 100; i++) {
		running = await cli("export", "status", cancelled.job.id);
		if (running.job.progress > 0 || running.job.status !== "queued") break;
		await page.waitForTimeout(100);
	}
	assert.equal(running.job.status, "running", JSON.stringify(running));
	await cli("export", "cancel", cancelled.job.id);
	assert.equal((await cli("export", "status", cancelled.job.id)).job.status, "cancelled");
	await page.waitForTimeout(1000);
	await assert.rejects(access(cancelled.job.options.outputPath));
	console.log("PASS: cancellation during rendering, no published partial output");
	const gifPath = path.join(root, "exports", `agent-gif-${Date.now()}.gif`);
	const gif = await cli(
		"export",
		"start",
		project,
		"--output",
		gifPath,
		"--format",
		"gif",
		"--width",
		"640",
		"--height",
		"360",
		"--wait",
	);
	assert.equal(gif.job.status, "completed", JSON.stringify(gif));
	assert.equal((await readFile(gifPath)).subarray(0, 6).toString(), "GIF89a");
	console.log("PASS: CLI GIF job after a cancelled render");
	const editedProject = path.join(root, ".cache", `edited-${Date.now()}.recordarch`);
	await writeFile(
		editedProject,
		JSON.stringify({
			...persisted,
			editor: {
				...persisted.editor,
				wallpaper: "#224422",
				padding: 20,
				shadowIntensity: 0,
				clipRegions: [
					{
						id: "edited",
						startMs: 0,
						endMs: 500,
						sourceStartMs: 1000,
						speed: 2,
						muted: true,
					},
				],
			},
		}),
	);
	const editedPath = path.join(root, "exports", `edited-${Date.now()}.mp4`);
	const edited = await cli(
		"export",
		"start",
		editedProject,
		"--output",
		editedPath,
		"--width",
		"640",
		"--height",
		"360",
		"--fps",
		"15",
		"--wait",
	);
	assert.equal(edited.job.status, "completed", JSON.stringify(edited));
	const editedMeta = JSON.parse(
		(await exec("ffprobe", ["-v", "error", "-show_format", "-of", "json", editedPath])).stdout,
	);
	assert(Math.abs(Number(editedMeta.format.duration) - 0.5) < 0.1);
	const pixels = (
		await exec(
			"ffmpeg",
			[
				"-v",
				"error",
				"-i",
				editedPath,
				"-frames:v",
				"1",
				"-vf",
				"scale=160:90",
				"-f",
				"rawvideo",
				"-pix_fmt",
				"rgb24",
				"-",
			],
			{ encoding: "buffer" },
		)
	).stdout;
	assert([...pixels.subarray(0, 3)].every((v, i) => Math.abs(v - [34, 68, 34][i]) < 15));
	const audio = (
		await exec("ffmpeg", ["-v", "error", "-i", editedPath, "-vn", "-f", "f32le", "-"], {
			encoding: "buffer",
		})
	).stdout;
	let amplitude = 0;
	for (let i = 0; i < audio.length; i += 4)
		amplitude = Math.max(amplitude, Math.abs(audio.readFloatLE(i)));
	assert(amplitude < 0.002, `Muted export must be silent: ${amplitude}`);
	console.log("PASS: source offsets, speed, background edits, duration and muted audio");
	await page.reload();
	await page.waitForFunction(() =>
		[...document.querySelectorAll("video")].some((v) => v.videoWidth === 640),
	);
	assert.equal((await cli("project", "list")).entries[0].path, project);
	console.log("PASS: latest project reopened");
	await page.screenshot({ path: path.join(root, "editor.png") });
	assert.deepEqual(errors, []);
	console.log("PASS: no uncaught browser errors");
} finally {
	await browser.close();
}
