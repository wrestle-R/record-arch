// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api0={
hudOverlaySetIgnoreMouse: (ignore) => {
    ipcRenderer.send("hud-overlay-set-ignore-mouse", ignore);
},
hudOverlaySetSourceSelectionActive: (active) => {
    ipcRenderer.send("hud-overlay-set-source-selection-active", active);
},
hudOverlayDrag: (phase, screenX, screenY) => {
    ipcRenderer.send("hud-overlay-drag", phase, screenX, screenY);
},
hudOverlayHide: () => {
    ipcRenderer.send("hud-overlay-hide");
},
hudOverlayClose: () => {
    ipcRenderer.send("hud-overlay-close");
},
getEditorMode: () => ipcRenderer.invoke("get-editor-mode"),
onEditorModeChanged: (callback) => {
    const listener = (_event, inEditor) => callback(inEditor);
    ipcRenderer.on("editor-mode-changed", listener);
    return () => ipcRenderer.removeListener("editor-mode-changed", listener);
},
hudOverlayRendererReady: () => {
    ipcRenderer.send("hud-overlay-renderer-ready");
},
hudOverlaySetWebcamPreviewVisible: (visible) => {
    ipcRenderer.send("hud-overlay-set-webcam-preview-visible", visible);
},
getHudOverlayCaptureProtection: () => {
    return ipcRenderer.invoke("get-hud-overlay-capture-protection");
},
getHudOverlayMousePassthroughSupported: () => {
    return ipcRenderer.invoke("get-hud-overlay-mouse-passthrough-supported");
},
setHudOverlayCaptureProtection: (enabled) => {
    return ipcRenderer.invoke("set-hud-overlay-capture-protection", enabled);
}
};
