// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api2={
nativeVideoExportWriteFrames: (sessionId, frameDataList) => {
    ensureNativeVideoExportWriteResultListener();
    return new Promise((resolve) => {
        const requestId = counters.nextNativeVideoExportWriteRequestId++;
        nativeVideoExportWriteRequests.set(requestId, {
            sessionId,
            resolve,
        });
        ipcRenderer.send("native-video-export-write-frames-async", {
            sessionId,
            requestId,
            frameDataList,
        });
    });
},
nativeVideoExportFinish: (sessionId, options) => {
    return ipcRenderer
        .invoke("native-video-export-finish", sessionId, options)
        .then((result) => {
        settleNativeVideoExportPendingRequests(sessionId, result?.success
            ? { success: true }
            : {
                success: false,
                error: typeof result?.error === "string"
                    ? result.error
                    : "Native video export session finished before all frame writes settled.",
            });
        return result;
    });
},
nativeVideoExportCancel: (sessionId) => {
    return ipcRenderer.invoke("native-video-export-cancel", sessionId).finally(() => {
        settleNativeVideoExportPendingRequests(sessionId, {
            success: false,
            error: "Native video export session was cancelled",
        });
    });
},
muxExportedVideoAudio: (videoData, options) => {
    return ipcRenderer.invoke("mux-exported-video-audio", videoData, options);
},
muxExportedVideoAudioFromPath: (videoPath, options) => {
    return ipcRenderer.invoke("mux-exported-video-audio-from-path", videoPath, options);
},
openExportStream: (options) => {
    return ipcRenderer.invoke("export-stream-open", options);
},
writeExportStreamChunk: (streamId, position, chunk) => {
    return ipcRenderer.invoke("export-stream-write", streamId, position, chunk);
},
closeExportStream: (streamId, options) => {
    return ipcRenderer.invoke("export-stream-close", streamId, options);
},
finalizeExportedVideo: (payload) => {
    return ipcRenderer.invoke("finalize-exported-video", payload);
},
discardExportedTemp: (tempPath) => {
    return ipcRenderer.invoke("discard-exported-temp", tempPath);
},
getVideoAudioFallbackPaths: (videoPath) => {
    return ipcRenderer.invoke("get-video-audio-fallback-paths", videoPath);
},
getSources: async (opts) => {
    return await ipcRenderer.invoke("get-sources", opts);
}
};
