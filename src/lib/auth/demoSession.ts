import type { User } from "@supabase/supabase-js";

// Temporary local UI account. Never creates a Supabase session or cloud access token.
export const demoLoginEnabled = import.meta.env.DEV;
const key = "recordly.demo-session";
const event = "recordly-demo-session-changed";
export const demoUser: User = {
	id: "recordly-local-demo",
	email: "test@email.com",
	aud: "local-demo",
	app_metadata: {},
	user_metadata: { full_name: "Test User" },
	created_at: "2026-09-23T00:00:00.000Z",
};
export function hasDemoSession() {
	return demoLoginEnabled && typeof window !== "undefined" && sessionStorage.getItem(key) === "1";
}
export function setDemoSession(active: boolean) {
	if (!demoLoginEnabled) return;
	if (active) sessionStorage.setItem(key, "1");
	else sessionStorage.removeItem(key);
	window.dispatchEvent(new Event(event));
}
export function subscribeDemoSession(listener: () => void) {
	window.addEventListener(event, listener);
	return () => window.removeEventListener(event, listener);
}
