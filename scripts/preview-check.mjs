import { chromium } from "@playwright/test";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { writeFile } from "node:fs/promises";
import assert from "node:assert/strict";
const exec = promisify(execFile);
const browser = await chromium.launch({
	headless: true,
	args: [
		"--no-sandbox",
		"--use-gl=angle",
		"--use-angle=swiftshader",
		"--enable-unsafe-swiftshader",
	],
});
try {
	const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
	await page.goto("http://localhost:1420");
	await page.waitForFunction(() => document.querySelector("video")?.videoWidth === 640);
	const canvas = page.locator("canvas:visible").first();
	await canvas.waitFor({ state: "visible" });
	let colors = 0;
	for (let attempt = 0; attempt < 12; attempt++) {
		await writeFile("/tmp/record-arch-preview.png", await canvas.screenshot());
		const { stdout } = await exec(
			"ffmpeg",
			[
				"-v",
				"error",
				"-i",
				"/tmp/record-arch-preview.png",
				"-vf",
				"scale=320:180",
				"-f",
				"rawvideo",
				"-pix_fmt",
				"rgb24",
				"-",
			],
			{ encoding: "buffer", maxBuffer: 1024 * 1024 },
		);
		colors = 0;
		for (let i = 0; i < stdout.length; i += 3) {
			const r = stdout[i],
				g = stdout[i + 1],
				b = stdout[i + 2];
			if ((r > 180 && g < 80 && b < 80) || (g > 180 && r < 80 && b < 80)) colors++;
		}
		if (colors > 1000) break;
		await page.waitForTimeout(500);
	}
	assert(
		colors > 1000,
		`Preview did not show synthetic red/green footage: ${colors} colored pixels`,
	);
	console.log("PASS: composed editor preview contains decoded video pixels");
} finally {
	await browser.close();
}
