from pathlib import Path
p=Path('src/components/Canvas.svelte');s=p.read_text(encoding='utf-8-sig');s=s.replace("let frame=$state(),selection=", "let stage=$state(),available=$state({w:800,h:500}),imageSize=$state({w:1600,h:1200});let fit=$derived(Math.min(available.w*.88/imageSize.w,available.h*.90/imageSize.h));\n let frame=$state(),selection=")
s=s.replace("function point(e)","$effect(()=>{if(!stage)return;const observer=new ResizeObserver(()=>available={w:stage.clientWidth,h:stage.clientHeight});observer.observe(stage);return()=>observer.disconnect();});\n function point(e)")
s=s.replace('<div class={`canvas-stage ${studio.background}`}', '<div bind:this={stage} class={`canvas-stage ${studio.background}`}')
s=s.replace('class="photo-wrap" style:transform=', 'class="photo-wrap" style:width={`${imageSize.w*fit}px`} style:height={`${imageSize.h*fit}px`} style:transform=')
s=s.replace('alt={asset().name} draggable="false"', 'alt={asset().name} onload={e=>imageSize={w:e.currentTarget.naturalWidth,h:e.currentTarget.naturalHeight}} draggable="false"')
p.write_text(s,encoding='utf-8')
