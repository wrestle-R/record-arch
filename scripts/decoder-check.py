import socket,json,base64,struct
path='/tmp/record-arch-integration.sock'
def rpc(channel,*args):
 s=socket.socket(socket.AF_UNIX);s.settimeout(10);s.connect(path);s.sendall((json.dumps({'channel':channel,'args':args})+'\n').encode());data=b''
 while not data.endswith(b'\n'):data+=s.recv(65536)
 s.close();return json.loads(data)
source=rpc('get-current-video-path')['path']
opened=rpc('arch-decoder-open',source,0,24);print(opened,flush=True)
for i in range(3):
 result=rpc('arch-decoder-next',opened['sessionId']);print(i,result.get('success'),len(result.get('dataUrl','')),flush=True)
print(rpc('arch-decoder-close',opened['sessionId']))
