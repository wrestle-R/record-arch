// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api7={
onWhisperSmallModelDownloadProgress: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("whisper-small-model-download-progress", listener);
    return () => ipcRenderer.removeListener("whisper-small-model-download-progress", listener);
},
generateAutoCaptions: (options) => {
    return ipcRenderer.invoke("generate-auto-captions", options);
},
setCurrentVideoPath: (path, options) => {
    return ipcRenderer.invoke("set-current-video-path", path, options);
},
setCurrentRecordingSession: (session, options) => {
    return ipcRenderer.invoke("set-current-recording-session", session, options);
},
onRecordingSessionChanged: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("recording-session-changed", listener);
    return () => ipcRenderer.removeListener("recording-session-changed", listener);
},
getCurrentRecordingSession: () => {
    return ipcRenderer.invoke("get-current-recording-session");
},
getCurrentVideoPath: () => {
    return ipcRenderer.invoke("get-current-video-path");
},
clearCurrentVideoPath: () => {
    return ipcRenderer.invoke("clear-current-video-path");
},
getRecordingThumbnail: (filePath) => ipcRenderer.invoke("get-recording-thumbnail", filePath),
finishRecordingImport: (keepPath, commit) => ipcRenderer.invoke("finish-recording-import", keepPath, commit),
cancelRecordingImport: () => ipcRenderer.invoke("cancel-recording-import"),
getProjectPreview: (projectPath) => ipcRenderer.invoke("get-project-preview", projectPath)
};
