import ts from 'typescript';
import fs from 'node:fs';
const source = ts.transpileModule(fs.readFileSync('reference/preload.ts','utf8'), {compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText;
const parsed=ts.createSourceFile('preload.js',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.JS);
const call=parsed.statements.find(s=>ts.isExpressionStatement(s)&&ts.isCallExpression(s.expression)&&s.expression.expression.getText(parsed).includes('exposeInMainWorld'));
const properties=call.expression.arguments[1].properties;
const printer=ts.createPrinter();
const runtimeNames=['nativeVideoExportWriteRequests','nextNativeVideoExportWriteRequestId','nativeVideoExportWriteResultListenerAttached','ensureNativeVideoExportWriteResultListener','settleNativeVideoExportPendingRequests'];
let prefix=source.slice(source.indexOf('const nativeVideoExportWriteRequests'),source.indexOf('contextBridge.exposeInMainWorld'));
prefix=prefix.replace('let nextNativeVideoExportWriteRequestId = 1;','').replace('let nativeVideoExportWriteResultListenerAttached = false;','');
for(const name of runtimeNames.slice(1,3))prefix=prefix.replaceAll(new RegExp(`\\b${name}\\b`,'g'),`counters.${name}`);
fs.writeFileSync('src/desktop/generated/runtime.ts','// @ts-nocheck\n// Generated from the attributed upstream compatibility interface.\nimport { ipcRenderer } from "../transport";\nexport const counters={nextNativeVideoExportWriteRequestId:1,nativeVideoExportWriteResultListenerAttached:false};\n'+prefix+'\nexport { nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests };\n');
const imports=[];
for(let i=0;i<properties.length;i+=12){
 const name=`api${i/12}`;let body=properties.slice(i,i+12).map(p=>printer.printNode(ts.EmitHint.Unspecified,p,parsed)).join(',\n');
 for(const key of runtimeNames.slice(1,3))body=body.replaceAll(new RegExp(`\\b${key}\\b`,'g'),`counters.${key}`);
 fs.writeFileSync(`src/desktop/generated/${name}.ts`,'// @ts-nocheck\n// Generated compatibility methods. Edit transport or the Rust handlers.\nimport { ipcRenderer } from "../transport";\nimport {counters,nativeVideoExportWriteRequests,ensureNativeVideoExportWriteResultListener,settleNativeVideoExportPendingRequests} from "./runtime";\nexport const '+name+'={\n'+body+'\n};\n');imports.push(name);
}
fs.writeFileSync('src/desktop/generated/index.ts',imports.map(n=>`import {${n}} from "./${n}";`).join('\n')+'\nexport const desktopApi={'+imports.map(n=>'...'+n).join(',')+'};\n');
fs.writeFileSync('docs/api-surface.json',JSON.stringify({methods:properties.map(p=>p.name.getText(parsed)),channels:[...new Set([...source.matchAll(/ipcRenderer\.(?:invoke|send|sendSync)\(\s*"([^"]+)"/g)].map(m=>m[1]))]},null,2)+'\n');
console.log(`Generated ${properties.length} compatibility methods`);
