import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/plugin-dialog';
import { frontendCommand } from './windowCommands';

type Callback = (...args: any[]) => void;
const listeners = new Map<string, Set<Callback>>();
const subscriptions = new Map<string, Promise<() => void>>();
let backend: {url:string;token:string} | null = null;
let settings: Record<string,unknown> = {};
function serialize(value:any):any {
  if(value instanceof ArrayBuffer || ArrayBuffer.isView(value)) {
    const bytes=value instanceof ArrayBuffer?new Uint8Array(value):new Uint8Array(value.buffer,value.byteOffset,value.byteLength);
    let text=''; for(let i=0;i<bytes.length;i+=8192)text+=String.fromCharCode(...bytes.subarray(i,i+8192));
    return {__bytes:btoa(text)};
  }
  if(Array.isArray(value))return value.map(serialize);
  if(value && typeof value==='object')return Object.fromEntries(Object.entries(value).map(([k,v])=>[k,serialize(v)]));
  return value??null;
}
export async function rpc(channel:string,...args:any[]):Promise<any> {
  if(isTauri())return invoke('command',{channel,args:serialize(args)});
  if(!backend)throw new Error('Start the Record Arch backend to use desktop features.');
  const response=await fetch(`${backend.url}/rpc`,{method:'POST',headers:{'Content-Type':'application/json','X-Record-Arch-Token':backend.token},body:JSON.stringify({channel,args:serialize(args)})});
  if(!response.ok)throw new Error(`Backend returned ${response.status}`);
  return response.json();
}
export function emitLocal(channel:string,...args:any[]) {
  for(const callback of listeners.get(channel)??[])callback({},...args);
}
async function picker(channel:string,args:any[]):Promise<any> {
  if(!isTauri())return {success:false,error:'Native file dialogs require the desktop app. Use the path field in the browser preview.'};
  const extensions=channel==='open-audio-file-picker'?['mp3','wav','ogg','m4a','flac']:channel.includes('whisper')?[]:['mp4','mov','webm','mkv','recordarch','recordly'];
  const selected=await open({multiple:false,filters:extensions.length?[{name:'Media and projects',extensions}]:undefined});
  if(!selected)return {success:false,canceled:true};
  if(channel.includes('whisper')||channel==='open-audio-file-picker')return {success:true,path:selected};
  if(/\.(recordarch|recordly)$/i.test(selected)){const result=await rpc('open-project-file-at-path',selected);return {...result,kind:'project'};}
  const result=await rpc('arch-import',selected);return {...result,kind:'video'};
}
export const ipcRenderer = {
  async invoke(channel:string,...args:any[]):Promise<any> {
    const frontend=await frontendCommand(channel,args);
    if(frontend.handled)return frontend.value;
    if(channel==='get-asset-base-path')return new URL('/',window.location.href).toString();
    if(channel==='list-asset-directory'){const manifest=await fetch('/asset-manifest.json').then(r=>r.json());return {success:true,entries:manifest[args[0]]??[],files:(manifest[args[0]]??[]).map((f:any)=>f.name)};}
    if(channel.startsWith('open-') && channel.endsWith('-picker') || channel==='load-project-file')return picker(channel,args);
    if(channel==='save-exported-video'){
      const path=isTauri()?await save({defaultPath:args[1]}):undefined;
      if(isTauri()&&!path)return {success:false,canceled:true};
      return rpc('write-exported-video-to-path',args[0],path??`${(await rpc('arch-doctor')).storage}/exports/${args[1]}`,args[2]);
    }
    if(channel==='finalize-exported-video' && !args[0].outputPath && isTauri()){
      const outputPath=await save({defaultPath:args[0].fileName??'export.mp4'});
      if(!outputPath)return {success:false,canceled:true};
      args[0]={...args[0],outputPath};
    }
    if(channel==='get-update-status-summary')return {status:'idle',availableVersion:null};
    if(channel==='get-current-update-toast-payload'||channel==='get-pending-auth-callback-url')return null;
    return rpc(channel,...args);
  },
  send(channel:string,...args:any[]) {
    if(['set-has-unsaved-changes','hud-overlay-renderer-ready'].includes(channel)){window.dispatchEvent(new CustomEvent(channel,{detail:args}));return;}
    void this.invoke(channel,...args).catch(error=>console.error(channel,error));
  },
  sendSync(channel:string,...args:any[]):any {
    if(channel==='app-settings:get')return {success:true,value:settings[args[0]]??null};
    if(channel==='app-settings:set'){settings[args[0]]=args[1];localStorage.setItem('record-arch.settings',JSON.stringify(settings));void rpc('arch-set-setting',...args).catch(console.error);return {success:true};}
    throw new Error(`Synchronous command unavailable: ${channel}`);
  },
  on(channel:string,callback:Callback) {
    if(!listeners.has(channel))listeners.set(channel,new Set());listeners.get(channel)!.add(callback);
    if(isTauri()&&!subscriptions.has(channel))subscriptions.set(channel,listen<any[]>(channel,e=>emitLocal(channel,...(Array.isArray(e.payload)?e.payload:[e.payload]))));
  },
  removeListener(channel:string,callback:Callback) {
    const callbacks=listeners.get(channel);callbacks?.delete(callback);
    if(callbacks?.size===0){void subscriptions.get(channel)?.then(stop=>stop());subscriptions.delete(channel);listeners.delete(channel);}
  },
};
export async function connectBackend(){
  try {
    if(!isTauri())backend=await fetch('/__record-arch-backend').then(r=>{if(!r.ok)throw new Error('Backend unavailable');return r.json();});
    settings=await rpc('arch-settings');
    if(!isTauri())window.setInterval(()=>void rpc('arch-events').then(events=>events.forEach((e:any)=>emitLocal(e.channel,...e.args))).catch(()=>{}),300);
  }catch{settings=JSON.parse(localStorage.getItem('record-arch.settings')??'{}');}
}
