// Pixi's browser adapter reads navigator while modules initialize in Node tests.
if (typeof navigator === 'undefined') {
  Object.defineProperty(globalThis,'navigator',{value:{userAgent:'RecordArchTest'},configurable:true});
}
