# EDGE Studio 2 — migration record

## Scope and decisions
Source: `D:/AI/AI应用项目/GVPAK STUDIO 2`. Destination: this project.
User explicitly chose a full Svelte rewrite and an independent data directory. The original project is not edited. Original projects, credentials and cloud jobs were not copied.

## Implemented mapping
| Original capability | EDGE destination |
|---|---|
| Projects, create, rename, image grid, multiselect, stacked covers with scroll | Library.svelte |
| Files/folders/native picker, browser drop including folders, duplicate skip | studio.svelte.js + transport.rs + retained core.rs |
| Exposure, contrast, saturation, temperature, exact inputs | Inspector.svelte, Rust preview |
| Free crop, ratios, rotation, both flips, undo/redo, pan/zoom, comparison | Canvas.svelte + Inspector.svelte |
| Tool/mode selection, Q/E, Ctrl+1–4, two-level context tool ring | App.svelte |
| Coverflow selection, shared multiselect, wheel and arrow navigation | Coverflow.svelte |
| Locked original and optional global color adjustment layer with opacity | Inspector.svelte + core.rs |
| Real RGB histogram | Histogram.svelte |
| Version save, restore, original restore and generated/local review | Inspector.svelte + Review.svelte |
| Trash and restore, immutable imported originals | Library.svelte + core.rs |
| WebP/JPEG/PNG, exact size, crop/pad, quality, byte limits, unique names, report, cancel | Export.svelte + core.rs |
| Google credentials, official generation, per-request approval, model/resolution/ratio | Settings.svelte + Generate.svelte + native Rust commands |
| USD/CNY, manual exchange rate, token estimates, model catalog/override warning | cost.ts + Generate.svelte + Settings.svelte |
| Generated version review, cost metadata, adopt for edits/exports | Review.svelte |
| DeepSeek planning, mixed local/Google Batch/local jobs, isolated output folders | Assistant.svelte + assistant.rs + hybrid.rs |
| Persistent job recovery, cancellation, output folders, completion notices | Assistant.svelte + SSE transport + hybrid.rs |
| Optional desktop notifications | Settings.svelte + studio.svelte.js |
| Reorderable/editable local steps, sample-first apply, saved free presets | Assistant.svelte |
| Color, rotate, flip, crop, resize/pad, flatten, blur, sharpen, OpenCV denoise, center, deskew | retained assistant.rs/opencv_worker.py |
| Data-root switching with environment and active-job checks | Settings.svelte + retained Store |

## Coverflow parity
Reimplemented from `integrations/aistudio-coverflow/src/CoverFlow.tsx`, not replaced by a different carousel. Source reference saved in `docs/CoverFlow-reference.tsx` for comparison only; no React dependency is built.
- Perspective 1000 px; centered frame translateZ 60 px, scale 1.06.
- Side yaw ±44 degrees; first offset 130 px, additional spacing 75 px, depth −40 px per offset.
- Side scale max(0.68, 0.95 − abs(offset) × 0.06); opacity max(0.25, 1 − abs(offset) × 0.18).
- Cull offsets beyond 7; wheel threshold ±35.
- Transform 350 ms cubic-bezier(0.16, 1, 0.3, 1); opacity 300 ms.
- Header opens project library; project page retains the −14-degree entry transition and scrollable stacked covers.
- EDGE colors, type, borders and Lucide controls replace the original warm/burgundy skin.

## Technical changes
Pure Svelte 5 frontend. React/JSX is not included in runtime. Three.js removed, spatial scene and Tour removed. CSS perspective remains solely for the explicitly retained Coverflow interaction.
All 39 original Rust commands are exposed by the local HTTP adapter. The pixel/store core is reused rather than substituted with browser-only mock functions. Progress and job completion are delivered through SSE. Native file/folder selection remains available via rfd. Browser drag/drop imports data through local HTTP without cloud upload.
HTTP commands require a custom request header and check browser origins. The service listens only on loopback.

## Verification boundaries
Original Rust tests: 18 passed. Real isolated HTTP/image checks: 23 passed (see verification/api-report.json).
No paid Google or DeepSeek requests were submitted during migration. Their original backend implementations were retained; new credentials are configured through Settings.
The original application's unimplemented future features are not claimed: multiple pixel layers, masks, brush/healing, PSD, and installer remain outside the source feature set.
