import { lazy,Suspense,useEffect,useState } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { rpc,ipcRenderer } from '@/desktop/transport';
import { RecorderPanel } from './RecorderPanel';
import './arch.css';
const Editor=lazy(()=>import('@/components/video-editor/EditorWindow'));
export default function ArchApp(){
  const recorderWindow=new URLSearchParams(location.search).get('windowType')==='recorder';
  const [recorder,setRecorder]=useState(recorderWindow);const [version,setVersion]=useState(0);
  useEffect(()=>{
    document.title='Record Arch';
    const openEditor=()=>{setVersion(v=>v+1);setRecorder(false);};
    const openRecorder=()=>{if(isTauri()&&!recorderWindow)void rpc('arch-show-recorder');else setRecorder(true);};
    const recordingComplete=()=>{if(!recorderWindow){setVersion(v=>v+1);setRecorder(false);if(isTauri())void getCurrentWindow().show();}};
    window.addEventListener('arch-open-editor',openEditor);window.addEventListener('arch-open-recorder',openRecorder);
    ipcRenderer.on('arch-recording-complete',recordingComplete);
    ipcRenderer.on('arch-open-editor',openEditor);
    return()=>{window.removeEventListener('arch-open-editor',openEditor);window.removeEventListener('arch-open-recorder',openRecorder);ipcRenderer.removeListener('arch-recording-complete',recordingComplete);ipcRenderer.removeListener('arch-open-editor',openEditor);};
  },[recorderWindow]);
  if(recorderWindow)return <RecorderPanel onClose={()=>{void getCurrentWindow().close();}}/>;
  return <main className="arch-app"><Suspense fallback={<div className="arch-boot"><span className="arch-brand">Record Arch</span><p>Opening your workspace…</p></div>}><Editor key={version}/></Suspense>{recorder&&<div className="arch-modal" role="dialog" aria-modal="true" aria-label="New recording" onClick={e=>{if(e.target===e.currentTarget)setRecorder(false);}}><RecorderPanel onClose={()=>setRecorder(false)}/></div>}</main>;
}
