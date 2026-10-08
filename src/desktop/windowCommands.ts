import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { rpc } from './transport';
export async function frontendCommand(channel:string,args:any[]):Promise<{handled:boolean;value?:unknown}>{
  if(['show-recording-hud','open-source-selector'].includes(channel)){
    window.dispatchEvent(new Event('arch-open-recorder'));return {handled:true,value:{success:true}};
  }
  if(['switch-to-editor','show-project-dashboard'].includes(channel)){
    window.dispatchEvent(new Event(channel==='switch-to-editor'?'arch-open-editor':'arch-open-library'));return {handled:true,value:{success:true}};
  }
  if(['hud-overlay-hide','hud-overlay-close'].includes(channel)){
    if(isTauri())await getCurrentWindow().hide();return {handled:true,value:{success:true}};
  }
  if(['open-projects-directory','open-recordings-folder','reveal-in-folder','open-external-url'].includes(channel))return {handled:false};
  if(['hud-overlay-set-ignore-mouse','hud-overlay-set-source-selection-active','hud-overlay-set-webcam-preview-visible','hud-overlay-drag','show-source-highlight','set-cursor-scale','set-recording-state','save-before-close-done'].includes(channel))return {handled:true,value:{success:true}};
  return {handled:false};
}
