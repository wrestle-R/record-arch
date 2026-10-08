import { useEffect,useState } from 'react';
import { Monitor,Record,X,Microphone,ArrowRight,WarningCircle } from '@phosphor-icons/react';
import { rpc } from '@/desktop/transport';
import { isTauri } from '@tauri-apps/api/core';
type Source={id:string;name:string;sourceType:string};
export function RecorderPanel({onClose}:{onClose:()=>void}){
  const [sources,setSources]=useState<Source[]>([]);const [source,setSource]=useState('');
  const [audio,setAudio]=useState(false);const [cursor,setCursor]=useState(false);
  const [busy,setBusy]=useState(false);const [recording,setRecording]=useState(false);const [error,setError]=useState('');
  useEffect(()=>{void rpc('arch-sources').then(result=>{if(!Array.isArray(result))throw new Error(result.error??'Could not list sources');setSources(result);setSource(result[0]?.id??'');}).catch(e=>setError(String(e)));},[]);
  async function start(){setBusy(true);setError('');try{
    if(isTauri())await rpc('arch-hide-recording-windows');
    const result=await rpc('arch-record-start',source,{systemAudio:audio,hideCursor:cursor});
    if(!result.success)throw new Error(result.error);setRecording(true);
  }catch(e){setError(String(e));if(isTauri())await rpc('arch-show-recording-windows');}finally{setBusy(false);}}
  async function stop(){setBusy(true);try{const result=await rpc('arch-record-stop');if(result.success===false)throw new Error(result.error);window.dispatchEvent(new Event('arch-open-editor'));onClose();}catch(e){setError(String(e));}finally{setBusy(false);}}
  return <section className="arch-recorder" aria-label="Recording setup"><header><span className="arch-brand-small"><Record size={22} weight="duotone"/>Record Arch</span><button aria-label="Close recording setup" onClick={onClose}><X size={20}/></button></header>
    <p className="arch-eyebrow">Capture something worth showing</p><h1>Ready when you are.</h1><p className="arch-muted">Choose your source. We’ll bring the take straight into your editor.</p>
    <label className="arch-field"><span><Monitor size={18}/>Capture source</span><select value={source} onChange={e=>setSource(e.target.value)} disabled={recording}>{sources.map(s=><option key={s.id} value={s.id}>{s.sourceType==='screen'?'Display':'Window region'} · {s.name}</option>)}</select></label>
    <label className="arch-toggle"><span><Microphone size={18}/>Include system audio</span><input type="checkbox" checked={audio} onChange={e=>setAudio(e.target.checked)} disabled={recording}/></label>
    <label className="arch-toggle"><span>Hide the original cursor</span><input type="checkbox" checked={cursor} onChange={e=>setCursor(e.target.checked)} disabled={recording}/></label>
    <aside className="arch-capture-note"><WarningCircle size={20}/><p>On Hyprland, recording controls hide during capture. Stop with your configured shortcut or <code>record-arch record stop</code>. Window regions stay fixed if you move a window.</p></aside>
    {error&&<p className="arch-error" role="alert">{error}</p>}
    <button className="arch-button arch-primary arch-record-start" onClick={()=>void(recording?stop():start())} disabled={busy||!source}><Record size={20} weight="fill"/>{busy?'Working…':recording?'Stop recording':'Start recording'}<ArrowRight size={18}/></button>
    <p className="arch-storage-note">Saved locally in ~/Record-Arch/recordings</p>
  </section>;
}
