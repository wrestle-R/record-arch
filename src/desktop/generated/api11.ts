// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api11={
ackAuthCallbackUrl: (url) => ipcRenderer.invoke("auth:ack-callback", url),
getPendingAuthCallbackUrl: () => ipcRenderer.invoke("auth:get-pending-callback"),
onAuthCallbackUrl: (callback) => {
    const listener = (_event, url) => callback(url);
    ipcRenderer.on("auth:callback", listener);
    return () => ipcRenderer.removeListener("auth:callback", listener);
},
revealInFolder: (filePath) => {
    return ipcRenderer.invoke("reveal-in-folder", filePath);
},
cloudShareUpload: (input) => ipcRenderer.invoke("cloud-share-upload", input),
cloudShareManage: (input) => ipcRenderer.invoke("cloud-share-manage", input),
cloudShareCancel: (uploadId) => ipcRenderer.invoke("cloud-share-cancel", uploadId),
onCloudShareProgress: (callback) => {
    const listener = (_event, progress) => callback(progress);
    ipcRenderer.on("cloud-share-progress", listener);
    return () => ipcRenderer.removeListener("cloud-share-progress", listener);
},
openRecordingsFolder: () => {
    return ipcRenderer.invoke("open-recordings-folder");
},
getRecordingsDirectory: () => {
    return ipcRenderer.invoke("get-recordings-directory");
},
chooseRecordingsDirectory: () => {
    return ipcRenderer.invoke("choose-recordings-directory");
},
getShortcuts: () => {
    return ipcRenderer.invoke("get-shortcuts");
}
};
