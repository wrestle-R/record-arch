// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api6={
getScreenRecordingPermissionStatus: () => {
    return ipcRenderer.invoke("get-screen-recording-permission-status");
},
openScreenRecordingPreferences: () => {
    return ipcRenderer.invoke("open-screen-recording-preferences");
},
openAccessibilityPreferences: () => {
    return ipcRenderer.invoke("open-accessibility-preferences");
},
saveExportedVideo: (videoData, fileName, captionSidecar) => {
    return ipcRenderer.invoke("save-exported-video", videoData, fileName, captionSidecar);
},
writeExportedVideoToPath: (videoData, outputPath, captionSidecar) => {
    return ipcRenderer.invoke("write-exported-video-to-path", videoData, outputPath, captionSidecar);
},
openVideoFilePicker: (options) => {
    return ipcRenderer.invoke("open-video-file-picker", options);
},
openAudioFilePicker: () => {
    return ipcRenderer.invoke("open-audio-file-picker");
},
openWhisperExecutablePicker: () => {
    return ipcRenderer.invoke("open-whisper-executable-picker");
},
openWhisperModelPicker: () => {
    return ipcRenderer.invoke("open-whisper-model-picker");
},
getWhisperSmallModelStatus: () => {
    return ipcRenderer.invoke("get-whisper-small-model-status");
},
downloadWhisperSmallModel: () => {
    return ipcRenderer.invoke("download-whisper-small-model");
},
deleteWhisperSmallModel: () => {
    return ipcRenderer.invoke("delete-whisper-small-model");
}
};
