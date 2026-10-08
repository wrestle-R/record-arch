import { useState } from "react";
import { FilmStrip, FolderOpen, Record, ArrowRight } from "@phosphor-icons/react";
import { rpc } from "@/desktop/transport";
import { isTauri } from "@tauri-apps/api/core";
export function EmptyEditor() {
	const [path, setPath] = useState("");
	const [error, setError] = useState("");
	const [busy, setBusy] = useState(false);
	async function openVideo() {
		setBusy(true);
		setError("");
		try {
			const result = isTauri()
				? await window.electronAPI.openVideoFilePicker({ includeProjects: true })
				: await rpc("arch-import", path);
			if (result.canceled) return;
			if (!result.success) throw new Error(result.error ?? "Could not open video");
			window.dispatchEvent(new Event("arch-open-editor"));
		} catch (e) {
			setError(String(e));
		} finally {
			setBusy(false);
		}
	}
	return (
		<section className="arch-empty" aria-labelledby="empty-title">
			<div className="arch-empty-symbol">
				<FilmStrip size={38} weight="duotone" />
			</div>
			<p className="arch-eyebrow">Your next good take starts here</p>
			<h1 id="empty-title">
				A little footage.
				<br />
				<span>A great story.</span>
			</h1>
			<p className="arch-empty-description">
				Bring in a recording. Find your focus. Make it yours with
				<br />
				smooth zooms, cursor effects, and a frame that feels right.
			</p>
			<div className="arch-empty-actions">
				<button
					className="arch-button arch-primary"
					onClick={() => void openVideo()}
					disabled={busy}
				>
					<FolderOpen size={18} />
					Open a video <kbd>Ctrl O</kbd>
				</button>
				<button
					className="arch-button"
					onClick={() => window.dispatchEvent(new Event("arch-open-recorder"))}
				>
					<Record size={18} />
					New recording
				</button>
			</div>
			{!isTauri() && (
				<form
					className="arch-path-form"
					onSubmit={(e) => {
						e.preventDefault();
						void openVideo();
					}}
				>
					<input
						aria-label="Local video path"
						placeholder="Absolute path to a local video"
						value={path}
						onChange={(e) => setPath(e.target.value)}
					/>
					<button aria-label="Import local video" disabled={!path || busy}>
						<ArrowRight size={18} />
					</button>
				</form>
			)}
			{error && (
				<p role="alert" className="arch-error">
					{error}
				</p>
			)}
			<div className="arch-workflow">
				<span>01 &nbsp; Import or record</span>
				<span>02 &nbsp; Shape your story</span>
				<span>03 &nbsp; Export and share</span>
			</div>
		</section>
	);
}
