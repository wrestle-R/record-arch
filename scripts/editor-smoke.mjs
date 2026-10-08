import {chromium} from '@playwright/test';
const browser=await chromium.launch({headless:true,args:['--no-sandbox','--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
const page=await browser.newPage({viewport:{width:1440,height:960}});
page.on('pageerror',e=>console.log('PAGE ERROR',e.message));
page.on('console',m=>{if(m.type()==='error'||m.type()==='warning')console.log(m.type(),m.text().slice(0,200));});
await page.goto('http://localhost:1420');
await page.locator('video').first().waitFor({state:'attached',timeout:60000});
await page.waitForFunction(()=>[...document.querySelectorAll('video')].some(v=>v.videoWidth>0),{},{timeout:60000});
console.log((await page.locator('body').innerText()).slice(-2000));
await page.screenshot({path:'/tmp/record-arch-editor.png'});
const result=await page.evaluate(async()=>{
 const {rpc,connectBackend}=await import('/src/desktop/transport.ts'); await connectBackend();
 const {NativeExporter}=await import('/src/arch/NativeExporter.ts');
 const source=await rpc('get-current-video-path');const url=await rpc('get-local-media-url',source.path);
 const exporter=new NativeExporter({videoUrl:url.url,width:640,height:360,frameRate:24,bitrate:1500000,wallpaper:'#1b2a33',zoomRegions:[],showShadow:false,shadowIntensity:0,backgroundBlur:0,borderRadius:0,padding:10,cropRegion:{x:0,y:0,width:1,height:1},showCursor:false,annotationRegions:[],clipRegions:[{id:'clip-1',startMs:0,endMs:1000,sourceStartMs:0,speed:1}]});
 return exporter.export();
});
console.log('EXPORT RESULT',JSON.stringify(result));
await browser.close();
