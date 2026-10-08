// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api5={
setRecordingState: (recording) => {
    return ipcRenderer.invoke("set-recording-state", recording);
},
setCursorScale: (scale) => {
    return ipcRenderer.invoke("set-cursor-scale", scale);
},
getCursorTelemetry: (videoPath) => {
    return ipcRenderer.invoke("get-cursor-telemetry", videoPath);
},
setCursorTelemetry: (videoPath, samples) => {
    return ipcRenderer.invoke("set-cursor-telemetry", videoPath, samples);
},
getSystemCursorAssets: () => {
    return ipcRenderer.invoke("get-system-cursor-assets");
},
onStopRecordingFromTray: (callback) => {
    const listener = () => callback();
    ipcRenderer.on("stop-recording-from-tray", listener);
    return () => ipcRenderer.removeListener("stop-recording-from-tray", listener);
},
onRecordingStateChanged: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("recording-state-changed", listener);
    return () => ipcRenderer.removeListener("recording-state-changed", listener);
},
onRecordingInterrupted: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("recording-interrupted", listener);
    return () => ipcRenderer.removeListener("recording-interrupted", listener);
},
onCursorStateChanged: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("cursor-state-changed", listener);
    return () => ipcRenderer.removeListener("cursor-state-changed", listener);
},
openExternalUrl: (url) => {
    return ipcRenderer.invoke("open-external-url", url);
},
getAccessibilityPermissionStatus: () => {
    return ipcRenderer.invoke("get-accessibility-permission-status");
},
requestAccessibilityPermission: () => {
    return ipcRenderer.invoke("request-accessibility-permission");
}
};
