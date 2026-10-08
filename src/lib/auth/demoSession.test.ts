import { afterEach, beforeEach, expect, it, vi } from "vitest";

beforeEach(() => {
	vi.resetModules();
	const values = new Map<string, string>();
	vi.stubGlobal("window", new EventTarget());
	vi.stubGlobal("sessionStorage", {
		getItem: (key: string) => values.get(key) ?? null,
		setItem: (key: string, value: string) => values.set(key, value),
		removeItem: (key: string) => values.delete(key),
	});
});
afterEach(() => {
	vi.unstubAllEnvs();
	vi.unstubAllGlobals();
});

it("notifies local account changes and stores only a session marker", async () => {
	vi.stubEnv("DEV", true);
	const { setDemoSession, hasDemoSession, subscribeDemoSession } = await import("./demoSession");
	const listener = vi.fn();
	const unsubscribe = subscribeDemoSession(listener);
	expect(hasDemoSession()).toBe(false);
	setDemoSession(true);
	expect(hasDemoSession()).toBe(true);
	expect(sessionStorage.getItem("recordly.demo-session")).toBe("1");
	setDemoSession(false);
	expect(hasDemoSession()).toBe(false);
	expect(listener).toHaveBeenCalledTimes(2);
	unsubscribe();
	setDemoSession(true);
	expect(listener).toHaveBeenCalledTimes(2);
});

it("cannot enable the local demo account in production", async () => {
	vi.stubEnv("DEV", false);
	sessionStorage.setItem("recordly.demo-session", "1");
	const { setDemoSession, hasDemoSession, demoLoginEnabled } = await import("./demoSession");
	setDemoSession(true);
	expect(demoLoginEnabled).toBe(false);
	expect(hasDemoSession()).toBe(false);
});
