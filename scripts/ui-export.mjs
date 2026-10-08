import { chromium } from "@playwright/test";
import { readdir } from "node:fs/promises";
import path from "node:path";
import assert from "node:assert/strict";
const directory = path.join(
	process.env.RECORD_ARCH_HOME ?? "/tmp/record-arch-integration",
	"exports",
);
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
page.on("pageerror", (e) => errors.push(e.message));
page.on("console", (m) => {
	if (m.type() === "error") console.log("CONSOLE ERROR", m.text().slice(0, 400));
});
async function waitOutput(extension, before) {
	for (let i = 0; i < 240; i++) {
		const files = await readdir(directory);
		const found = files.find((f) => f.endsWith(extension) && !before.has(f));
		if (found) return path.join(directory, found);
		await new Promise((r) => setTimeout(r, 500));
	}
	throw new Error(`No ${extension} output appeared. ${await page.locator("body").innerText()}`);
}
try {
	await page.goto("http://localhost:1420");
	await page.waitForFunction(() => document.querySelector("video")?.videoWidth > 0);
	await page.getByRole("button", { name: "Export", exact: true }).click();
	await page.getByText("MP4", { exact: true }).click();
	const beforeMp4 = new Set(await readdir(directory));
	await page.getByRole("button", { name: "Export Video", exact: true }).click();
	console.log("UI MP4", await waitOutput(".mp4", beforeMp4));
	await page.getByRole("button", { name: "Export", exact: true }).click();
	await page.getByText("GIF", { exact: true }).click();
	const beforeGif = new Set(await readdir(directory));
	await page.getByRole("button", { name: "Export GIF", exact: true }).click();
	console.log("UI GIF", await waitOutput(".gif", beforeGif));
	assert.deepEqual(errors, []);
} finally {
	await browser.close();
}
