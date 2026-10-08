import { desktopApi } from './generated';
import { connectBackend } from './transport';
export async function installDesktopApi(){
  await connectBackend();
  window.electronAPI=desktopApi as unknown as Window['electronAPI'];
}
