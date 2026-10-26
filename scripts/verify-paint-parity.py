from pathlib import Path
from PIL import Image
import base64,io,json,urllib.request
root=Path(__file__).resolve().parents[1]
state=json.loads((root/'backend/target/paint-test-state.json').read_text(encoding='utf-8-sig'))
fixtures=json.loads((root/'backend/target/paint-parity-fixtures.json').read_text())
report=[]
def preview(edit):
    data={'projectId':state['projectId'],'assetId':state['assetId'],'edit':edit}
    req=urllib.request.Request('http://127.0.0.1:8798/api/invoke/preview',json.dumps(data).encode(),{'Content-Type':'application/json','X-Edge-IPC':'1'})
    return Image.open(io.BytesIO(urllib.request.urlopen(req).read())).convert('RGBA')
for fixture in fixtures:
    base=list(preview({**fixture['edit'],'strokes':[]}).getdata())
    actual=list(preview(fixture['edit']).getdata());layer=base64.b64decode(fixture['rgba'])
    maximum=0;changed=0
    for i,(b,p) in enumerate(zip(base,actual)):
        rgba=layer[i*4:i*4+4];a=rgba[3]/255;old=b[3]/255;alpha=a+old*(1-a)
        expected=[int((rgba[c]*a+b[c]*old*(1-a))/alpha+.5) if alpha else 0 for c in range(3)]+[int(alpha*255+.5)]
        delta=max(abs(x-y) for x,y in zip(expected,p));maximum=max(maximum,delta);changed+=delta>0
    assert maximum<=1,(maximum,fixture['edit'])
    report.append({'rotation':fixture['edit']['rotation'],'crop':fixture['edit']['crop'],'flipH':fixture['edit']['flipH'],'flipV':fixture['edit']['flipV'],'maxChannelDifference':maximum,'differentPixels':changed})
(root/'docs/verification/paint-parity-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report))
