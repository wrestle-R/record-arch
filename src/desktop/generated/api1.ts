// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api1={
getAssetBasePath: async () => {
    return await ipcRenderer.invoke("get-asset-base-path");
},
listAssetDirectory: (relativeDir) => {
    return ipcRenderer.invoke("list-asset-directory", relativeDir);
},
readLocalFile: (filePath) => {
    return ipcRenderer.invoke("read-local-file", filePath);
},
generateWallpaperThumbnail: (filePath) => {
    return ipcRenderer.invoke("generate-wallpaper-thumbnail", filePath);
},
probeNativeVideoMetadata: (filePath) => {
    return ipcRenderer.invoke("probe-native-video-metadata", filePath);
},
getNativeExportCapabilities: () => {
    return ipcRenderer.invoke("get-native-export-capabilities");
},
getExportHardwareInfo: () => {
    return ipcRenderer.invoke("get-export-hardware-info");
},
nativeStaticLayoutExport: (options) => {
    return ipcRenderer.invoke("native-static-layout-export", options);
},
nativeStaticLayoutExportCancel: (sessionId) => {
    return ipcRenderer.invoke("native-static-layout-export-cancel", sessionId);
},
onNativeStaticLayoutExportProgress: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("native-static-layout-export-progress", listener);
    return () => ipcRenderer.removeListener("native-static-layout-export-progress", listener);
},
nativeVideoExportStart: (options) => {
    return ipcRenderer.invoke("native-video-export-start", options);
},
nativeVideoExportWriteFrame: (sessionId, frameData) => {
    ensureNativeVideoExportWriteResultListener();
    return new Promise((resolve) => {
        const requestId = counters.nextNativeVideoExportWriteRequestId++;
        nativeVideoExportWriteRequests.set(requestId, {
            sessionId,
            resolve,
        });
        ipcRenderer.send("native-video-export-write-frame-async", {
            sessionId,
            requestId,
            frameData,
        });
    });
}
};
