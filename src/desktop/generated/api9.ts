// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api9={
openProjectFileAtPath: (filePath) => {
    return ipcRenderer.invoke("open-project-file-at-path", filePath);
},
openProjectsDirectory: () => {
    return ipcRenderer.invoke("open-projects-directory");
},
installDownloadedUpdate: () => {
    return ipcRenderer.invoke("install-downloaded-update");
},
downloadAvailableUpdate: (installAfterDownload) => {
    return ipcRenderer.invoke("download-available-update", installAfterDownload);
},
deferDownloadedUpdate: (delayMs) => {
    return ipcRenderer.invoke("defer-downloaded-update", delayMs);
},
dismissUpdateToast: () => {
    return ipcRenderer.invoke("dismiss-update-toast");
},
skipUpdateVersion: () => {
    return ipcRenderer.invoke("skip-update-version");
},
getCurrentUpdateToastPayload: () => {
    return ipcRenderer.invoke("get-current-update-toast-payload");
},
getUpdateStatusSummary: () => {
    return ipcRenderer.invoke("get-update-status-summary");
},
getExperimentalUpdatesEnabled: () => {
    return ipcRenderer.invoke("get-experimental-updates-enabled");
},
setExperimentalUpdatesEnabled: (enabled) => {
    return ipcRenderer.invoke("set-experimental-updates-enabled", enabled);
},
previewUpdateToast: () => {
    return ipcRenderer.invoke("preview-update-toast");
}
};
