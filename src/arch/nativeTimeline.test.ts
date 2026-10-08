import { describe,it,expect } from 'vitest';
import { buildNativeTimeline } from './nativeTimeline';
describe('native timeline source mapping',()=>{
 it('preserves gaps, reordered source offsets, speeds, and clip muting',()=>{
   const timeline=buildNativeTimeline(10,[{id:'b',startMs:3000,endMs:4000,sourceStartMs:1000,speed:2,muted:true},{id:'a',startMs:0,endMs:2000,sourceStartMs:5000,speed:1}]);
   expect(timeline).toEqual([{startSec:5,endSec:7,speed:1,outputStart:0,outputEnd:2,muted:undefined},{startSec:1,endSec:3,speed:2,outputStart:3,outputEnd:4,muted:true}]);
 });
 it('removes trimmed intervals instead of retaining silent gaps',()=>{
   const timeline=buildNativeTimeline(4,undefined,[{id:'trim',startMs:1000,endMs:3000}]);
   expect(timeline.map(s=>[s.startSec,s.endSec,s.outputStart,s.outputEnd])).toEqual([[0,1,0,1],[3,4,1,2]]);
 });
});
