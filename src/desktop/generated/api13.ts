// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api13={
setRecordingPreferences: (prefs) => ipcRenderer.invoke("set-recording-preferences", prefs),
getCountdownDelay: () => ipcRenderer.invoke("get-countdown-delay"),
setCountdownDelay: (delay) => ipcRenderer.invoke("set-countdown-delay", delay),
finishRecordingStartup: () => ipcRenderer.invoke("finish-recording-startup"),
startCountdown: (seconds) => ipcRenderer.invoke("start-countdown", seconds),
cancelCountdown: () => ipcRenderer.invoke("cancel-countdown"),
getActiveCountdown: () => ipcRenderer.invoke("get-active-countdown"),
onCountdownTick: (callback) => {
    const listener = (_event, seconds) => callback(seconds);
    ipcRenderer.on("countdown-tick", listener);
    return () => ipcRenderer.removeListener("countdown-tick", listener);
}
};
