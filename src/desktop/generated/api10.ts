// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api10={
checkForAppUpdates: () => {
    return ipcRenderer.invoke("check-for-app-updates");
},
onUpdateToastStateChanged: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("update-toast-state", listener);
    return () => ipcRenderer.removeListener("update-toast-state", listener);
},
onUpdateReadyToast: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("update-ready-toast", listener);
    return () => ipcRenderer.removeListener("update-ready-toast", listener);
},
onMenuLoadProject: (callback) => {
    const listener = () => callback();
    ipcRenderer.on("menu-load-project", listener);
    return () => ipcRenderer.removeListener("menu-load-project", listener);
},
onMenuSaveProject: (callback) => {
    const listener = () => callback();
    ipcRenderer.on("menu-save-project", listener);
    return () => ipcRenderer.removeListener("menu-save-project", listener);
},
onMenuSaveProjectAs: (callback) => {
    const listener = () => callback();
    ipcRenderer.on("menu-save-project-as", listener);
    return () => ipcRenderer.removeListener("menu-save-project-as", listener);
},
getWindowChrome: () => ipcRenderer.invoke("get-window-chrome"),
onWindowChromeChanged: (callback) => {
    const listener = (_event, chrome) => callback(chrome);
    ipcRenderer.on("window-chrome-changed", listener);
    return () => ipcRenderer.removeListener("window-chrome-changed", listener);
},
getPlatform: () => {
    return ipcRenderer.invoke("get-platform");
},
isWindowFullscreen: () => {
    return ipcRenderer.invoke("get-window-fullscreen");
},
onWindowFullscreenChanged: (callback) => {
    const listener = (_event, isFullscreen) => callback(isFullscreen);
    ipcRenderer.on("window-fullscreen-changed", listener);
    return () => ipcRenderer.removeListener("window-fullscreen-changed", listener);
},
getLinuxWindowSystem: () => {
    return ipcRenderer.invoke("get-linux-window-system");
}
};
