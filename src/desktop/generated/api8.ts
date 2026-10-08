// @ts-nocheck
// Generated compatibility methods. Edit transport or the Rust handlers.
import { ipcRenderer } from "../transport";
import {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";
export const api8={
listRecordings: (includeSources) => ipcRenderer.invoke("list-recordings", includeSources),
setRecordingsRemoved: (paths, removed) => ipcRenderer.invoke("set-recordings-removed", paths, removed),
importRecording: (currentPath, recordingPath, webcam) => ipcRenderer.invoke("import-recording", currentPath, recordingPath, webcam),
deleteRecordingFile: (filePath) => {
    return ipcRenderer.invoke("delete-recording-file", filePath);
},
getLocalMediaUrl: (filePath) => {
    return ipcRenderer.invoke("get-local-media-url", filePath);
},
saveProjectFile: (projectData, suggestedName, existingProjectPath, thumbnailDataUrl) => {
    return ipcRenderer.invoke("save-project-file", projectData, suggestedName, existingProjectPath, thumbnailDataUrl);
},
saveProjectFileNamed: (projectData, projectName, thumbnailDataUrl, mode) => {
    return ipcRenderer.invoke("save-project-file-named", projectData, projectName, thumbnailDataUrl, mode);
},
loadProjectFile: () => {
    return ipcRenderer.invoke("load-project-file");
},
loadCurrentProjectFile: () => {
    return ipcRenderer.invoke("load-current-project-file");
},
getProjectsDirectory: () => {
    return ipcRenderer.invoke("get-projects-directory");
},
onProjectThumbnailReady: (callback) => {
    const listener = (_event, ready) => callback(ready);
    ipcRenderer.on("project-library-thumbnail-ready", listener);
    return () => ipcRenderer.removeListener("project-library-thumbnail-ready", listener);
},
listProjectFiles: () => {
    return ipcRenderer.invoke("list-project-files");
}
};
