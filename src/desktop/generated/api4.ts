// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api4={
stopNativeScreenRecording: () => {
    return ipcRenderer.invoke("stop-native-screen-recording");
},
recoverNativeScreenRecording: () => {
    return ipcRenderer.invoke("recover-native-screen-recording");
},
getLastNativeCaptureDiagnostics: () => {
    return ipcRenderer.invoke("get-last-native-capture-diagnostics");
},
pauseNativeScreenRecording: () => {
    return ipcRenderer.invoke("pause-native-screen-recording");
},
resumeNativeScreenRecording: () => {
    return ipcRenderer.invoke("resume-native-screen-recording");
},
pauseCursorCapture: (pausedAtMs) => {
    return ipcRenderer.invoke("pause-cursor-capture", pausedAtMs);
},
resumeCursorCapture: (resumedAtMs) => {
    return ipcRenderer.invoke("resume-cursor-capture", resumedAtMs);
},
startFfmpegRecording: (source) => {
    return ipcRenderer.invoke("start-ffmpeg-recording", source);
},
stopFfmpegRecording: () => {
    return ipcRenderer.invoke("stop-ffmpeg-recording");
},
storeRecordedVideo: (videoData, fileName) => {
    return ipcRenderer.invoke("store-recorded-video", videoData, fileName);
},
storeMicrophoneSidecar: (audioData, videoPath, options) => {
    return ipcRenderer.invoke("store-microphone-sidecar", audioData, videoPath, options);
},
getRecordedVideoPath: () => {
    return ipcRenderer.invoke("get-recorded-video-path");
}
};
