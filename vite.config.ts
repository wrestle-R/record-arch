import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
export default defineConfig({
	plugins: [
		react(),
		{
			name: "local-backend",
			configureServer(server) {
				server.middlewares.use("/__record-arch-backend", (_req, res) => {
					try {
						const file = path.join(
							process.env.RECORD_ARCH_HOME ?? path.join(os.homedir(), "Record-Arch"),
							".cache/backend.json",
						);
						res.setHeader("Content-Type", "application/json");
						res.end(fs.readFileSync(file));
					} catch {
						res.statusCode = 503;
						res.end("Backend not running");
					}
				});
			},
		},
	],
	resolve: { alias: { "@": path.resolve(__dirname, "src") } },
	server: {
		host: "127.0.0.1",
		port: 1420,
		strictPort: true,
		watch: { ignored: ["**/src-tauri/**"] },
	},
	clearScreen: false,
	build: { target: "esnext" },
});
