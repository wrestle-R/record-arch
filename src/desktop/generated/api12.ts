// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api12={
saveShortcuts: (shortcuts) => {
    return ipcRenderer.invoke("save-shortcuts", shortcuts);
},
getAppSetting: (key) => {
    const result = ipcRenderer.sendSync("app-settings:get", key);
    return result?.success ? (result.value ?? null) : null;
},
setAppSetting: (key, value) => {
    const result = ipcRenderer.sendSync("app-settings:set", key, value);
    return result?.success === true;
},
setHasUnsavedChanges: (hasChanges) => {
    ipcRenderer.send("set-has-unsaved-changes", hasChanges);
},
onRequestSaveBeforeClose: (callback) => {
    const listener = async (_event, requestId) => {
        let saved = false;
        try {
            saved = await callback();
        }
        catch {
            saved = false;
        }
        ipcRenderer.send("save-before-close-done", saved, requestId);
    };
    ipcRenderer.on("request-save-before-close", listener);
    return () => ipcRenderer.removeListener("request-save-before-close", listener);
},
isNativeWindowsCaptureAvailable: () => ipcRenderer.invoke("is-native-windows-capture-available"),
muxNativeWindowsRecording: (expectedDurationMs) => ipcRenderer.invoke("mux-native-windows-recording", expectedDurationMs),
hideOsCursor: () => ipcRenderer.invoke("hide-cursor"),
getAppVersion: () => ipcRenderer.invoke("app:getVersion"),
getAnnouncements: () => ipcRenderer.invoke("announcements:get"),
getRecordingPreferences: () => ipcRenderer.invoke("get-recording-preferences"),
getRecordingAudioLabConfig: () => ipcRenderer.invoke("get-recording-audio-lab-config")
};
