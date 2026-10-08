// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api3={
showRecordingHud: () => ipcRenderer.invoke("show-recording-hud"),
createProjectFile: (data, thumbnail) => ipcRenderer.invoke("create-project-file", data, thumbnail),
renameLibraryProject: (path, name) => ipcRenderer.invoke("rename-library-project", path, name),
trashProjectFiles: (paths) => ipcRenderer.invoke("trash-project-files", paths),
showProjectDashboard: () => ipcRenderer.invoke("show-project-dashboard"),
switchToEditor: () => {
    return ipcRenderer.invoke("switch-to-editor");
},
openSourceSelector: () => {
    return ipcRenderer.invoke("open-source-selector");
},
selectSource: (source) => {
    return ipcRenderer.invoke("select-source", source);
},
showSourceHighlight: (source) => {
    return ipcRenderer.invoke("show-source-highlight", source);
},
getSelectedSource: () => {
    return ipcRenderer.invoke("get-selected-source");
},
onSelectedSourceChanged: (callback) => {
    const listener = (_event, payload) => callback(payload);
    ipcRenderer.on("selected-source-changed", listener);
    return () => ipcRenderer.removeListener("selected-source-changed", listener);
},
startNativeScreenRecording: (source, options) => {
    return ipcRenderer.invoke("start-native-screen-recording", source, options);
}
};
