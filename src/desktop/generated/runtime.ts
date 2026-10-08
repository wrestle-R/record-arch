// @ts-nocheck
// Generated from the attributed upstream compatibility interface.
import { ipcRenderer } from "../transport";
export const counters={nextNativeVideoExportWriteRequestId:1,nativeVideoExportWriteResultListenerAttached:false};
const nativeVideoExportWriteRequests = new Map();


function ensureNativeVideoExportWriteResultListener() {
    if (counters.nativeVideoExportWriteResultListenerAttached) {
        return;
    }
    counters.nativeVideoExportWriteResultListenerAttached = true;
    ipcRenderer.on("native-video-export-write-frame-result", (_event, payload) => {
        if (typeof payload?.requestId !== "number") {
            return;
        }
        const pendingRequest = nativeVideoExportWriteRequests.get(payload.requestId);
        if (!pendingRequest) {
            return;
        }
        nativeVideoExportWriteRequests.delete(payload.requestId);
        pendingRequest.resolve({
            success: payload.success === true,
            error: payload.error,
        });
    });
}
function settleNativeVideoExportPendingRequests(sessionId, result) {
    for (const [requestId, pendingRequest] of nativeVideoExportWriteRequests.entries()) {
        if (pendingRequest.sessionId !== sessionId) {
            continue;
        }
        nativeVideoExportWriteRequests.delete(requestId);
        pendingRequest.resolve(result);
    }
}

export { nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests };
