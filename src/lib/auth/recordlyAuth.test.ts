import { afterEach, beforeEach, expect, it, vi } from "vitest";

const emailOtp = vi.hoisted(() => vi.fn(async (_options: unknown) => ({ error: null })));
const exchange = vi.hoisted(() => vi.fn(async (_code: string) => ({ error: null })));
const oauth = vi.hoisted(() =>
	vi.fn(async (_options: unknown) => ({
		data: { url: "https://auth.example.test/oauth" },
		error: null,
	})),
);
vi.mock("@supabase/supabase-js", () => ({
	createClient: () => ({
		auth: { exchangeCodeForSession: exchange, signInWithOAuth: oauth, signInWithOtp: emailOtp },
	}),
}));
beforeEach(() => {
	vi.resetModules();
	exchange.mockClear();
	oauth.mockClear();
	emailOtp.mockClear();
	vi.stubEnv("VITE_SUPABASE_URL", "https://auth.example.test");
	vi.stubEnv("VITE_SUPABASE_PUBLISHABLE_KEY", "test-key");
});
afterEach(() => {
	vi.unstubAllEnvs();
	vi.unstubAllGlobals();
});

it("exchanges a callback once when live and pending delivery overlap", async () => {
	const { completeAuthCallback } = await import("./recordlyAuth");
	const url = "recordly://auth/callback?code=one-time-code";
	await Promise.all([completeAuthCallback(url), completeAuthCallback(url)]);
	await completeAuthCallback(url);
	expect(exchange).toHaveBeenCalledExactlyOnceWith("one-time-code");
});

it("shows provider errors without attempting a code exchange", async () => {
	const { completeAuthCallback } = await import("./recordlyAuth");
	await expect(
		completeAuthCallback(
			"recordly://auth/callback?error=denied&error_description=Sign-in+cancelled",
		),
	).rejects.toThrow("Sign-in cancelled");
	expect(exchange).not.toHaveBeenCalled();
});

it("requests Microsoft's email scope and opens its OAuth URL externally", async () => {
	const openExternalUrl = vi.fn(async () => ({ success: true }));
	vi.stubGlobal("window", { electronAPI: { openExternalUrl } });
	const { signInWithSocial } = await import("./recordlyAuth");
	await signInWithSocial("azure");
	expect(oauth).toHaveBeenCalledWith(
		expect.objectContaining({
			provider: "azure",
			options: expect.objectContaining({ scopes: "email", skipBrowserRedirect: true }),
		}),
	);
	expect(openExternalUrl).toHaveBeenCalledExactlyOnceWith("https://auth.example.test/oauth");
});

for (const dev of [true, false]) {
	it(`requests an app-initiated email link with the ${dev ? "development" : "installed"} callback`, async () => {
		vi.stubEnv("DEV", dev);
		const { sendSignInLink } = await import("./recordlyAuth");
		await sendSignInLink("owner@example.test");
		expect(emailOtp).toHaveBeenCalledExactlyOnceWith({
			email: "owner@example.test",
			options: {
				emailRedirectTo: dev
					? "http://127.0.0.1:43821/auth/callback"
					: "recordly://auth/callback",
				shouldCreateUser: true,
			},
		});
	});
}

it("does not accept dashboard implicit session tokens as a PKCE callback", async () => {
	const { completeAuthCallback } = await import("./recordlyAuth");
	await expect(
		completeAuthCallback("recordly://auth/callback#access_token=test&refresh_token=test"),
	).rejects.toThrow("Request a new sign-in link from Recordly");
	expect(exchange).not.toHaveBeenCalled();
});
