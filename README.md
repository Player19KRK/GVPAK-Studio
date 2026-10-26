# GVPAK STUDIO

Svelte 5 image workspace, using the Rust image-processing, project, export and AI-task core migrated from GVPAK STUDIO 2.

## Start
Double-click `start.cmd`, then open http://127.0.0.1:8787/.
The bundled Windows x64 Rust executable serves the compiled interface and API. No Node/Rust installation is needed to run it. Denoise, automatic centering and deskew use an existing Python/OpenCV environment; Settings reports availability.

The Three.js spatial preview has been removed. Coverflow follows the original GVPAK STUDIO 2 motion and selection behavior, styled to match EDGE.

## Workflows
- Edit: local imports, color, free/ratio crop, transforms, radial painting tools, comparison, zoom/pan, layers, RGB histogram and saved versions.
- Projects: click EDGE or the project name. Create or rename projects, scroll stacked project covers, browse/multiselect images, trash/restore.
- Generate: configure the provider in Settings, enter a prompt, then click Generate directly and review/adopt the result. Clicking Generate submits a paid request; no additional approval checkbox is required. Estimated costs default to CNY using the exchange rate in Settings; saved currency preferences are retained.
- Export: selected images, WebP/JPEG/PNG, crop/pad, dimensions, quality and byte limit, native output folder, per-image report and cancellation.
- Process: automatic mixed tasks or editable local steps. Local steps can be sampled, compared, applied and saved as reusable presets. No API is required for manual local steps.
- Settings: Windows Credential Manager, model catalog, currency/exchange rate, local tool status, data directory, optional notifications and shortcuts.

## Independent storage
Default: `backend/studio-data/` under this app. Source projects, images and cloud jobs were not migrated. The old demo's `backend/data/` is left intact.
Credential service: `EDGE-STUDIO-dev` (or `EDGE-STUDIO-test`). Keys from GVPAK STUDIO 2 are not copied. Use Settings to configure them.
No paid API calls were made during migration verification.

## Development
`npm ci`
`npm run dev`
`npm run check`
`npm run build`
`cargo build --release --manifest-path backend/Cargo.toml`
`cargo test --manifest-path backend/Cargo.toml`

The original workspace has an isolated Rust toolchain at `.tools/`. Build commands used process-only RUSTUP_HOME and a pre-existing Cargo cache; global PATH was not changed.
API contract: `docs/command-contract.json`. Feature mapping and Coverflow parity: `docs/MIGRATION.md`. Pre-migration source backup: `docs/pre-migration-demo.zip`.

## Scope
The migrated source has one global adjustment layer, not Photoshop-style multilayer compositing. Masks, brush/healing, PSD, installer and other unimplemented source roadmap items are not represented as completed features.

## Workflow UX update
Editing drafts are automatically kept in browser storage, separately from saved versions. Save / Ctrl+S saves the current image. Export and Process default to selected images (or the current image) and expose an explicit scope switch. Workflow inputs survive switching pages. See `docs/UX-WORKFLOWS.md` for detailed changes and `docs/verification/workflow-ux.json` for this pass.

## Preview navigation
Project library and image grids support Ctrl + wheel thumbnail scaling (60–180%), remembered independently. The percentage button resets to 100%; minus/plus controls provide equivalent actions. Coverflow has a small All control with partial/all/none selection states. Hover or focus a project to open its translucent nine-cell preview; placement follows available viewport space. Scroll to browse further images and click a thumbnail to open it. Escape or moving away closes the preview.

## Canvas-first layout
The brand is now the text-only GVPAK STUDIO wordmark. Drag the divider above Coverflow to change its height, or the divider at the inspector's left edge to change its width. Sizes persist across reload. Double-click a divider to reset that dimension; arrow keys and Shift+arrows provide fine/coarse adjustments. Settings > Workspace provides numeric pixel values. Collapse Coverflow to its header, or use Focus to temporarily hide both panels. Reset panel layout restores compact defaults. Zoomed image size stays stable while panel resizing exposes more canvas; Fit mode fills the available space.

## Coverflow quick selection
With the mouse over expanded Coverflow, tap Space to toggle the current image. Hold Space and scroll to add the starting image and every crossed image to the selection. Reversing direction does not deselect images. Release Space, move the pointer out, press Escape or switch windows to stop. Text inputs, open dialogs and busy processing do not consume Space. The selection is shared with the grid, Export and Process.

Space selection uses one fixed intent per press: start on an unchecked image to select; release and press on a checked image to deselect. Scrolling remains one image per step. Revisiting an image during the same hold never toggles it. The intent changes only after releasing Space and pressing again.

While the pointer is over expanded Coverflow, hold Space and press Left/Right to select or deselect each visited image using the same fixed intent. Holding an arrow repeats one image per key event. Arrow keys and scrolling can be mixed; neither wraps at the ends. Release Space to return to ordinary arrow navigation.

## Responsive image preview
The visible image has priority: show its available thumbnail/cache immediately, load a 1600px preview source after navigation settles, and use WebGL2 for interactive color, crop and transform feedback. Rust replaces the interactive result after a 220ms pause and remains authoritative for saved/exported pixels. Neutral edits reuse the source without a second render. Comparison originals load only when Compare opens; histograms run after the image in an idle callback. Queued obsolete previews are discarded. Already-running native work finishes and may populate the cache.

The LRU targets 12 entries, 32 MiB encoded blobs and 128 MiB estimated decoded pixels; displayed entries are pinned. These are cache targets, not a cap on total browser/GPU memory. Settings > Preview acceleration offers CPU compatibility; unavailable GPU contexts fall back automatically. GPU resizing is an interactive approximation, especially after crops; the settled Rust render provides precise output.

Double-click the image canvas to restore centered fit at the app's 100% (not native 1:1 pixels). Holding Space over Coverflow displays Select or Deselect near its page counter, then fades after release. The intent stays fixed until release.

Verification: `node --test scripts/test-preview-cache.mjs scripts/test-coverflow-selection.mjs`; GPU/Rust fixtures in `docs/verification/gpu-qa`; measured results in `docs/verification/gpu-preview-report.json`.

Image grid: click an image to toggle selection; double-click to edit. Keyboard Space toggles selection and Enter opens the editor. Project peek dismisses immediately when the pointer leaves its card, panel and narrow connecting gap; there is no close timer.

## Assistant workflow presets
Process > Assistant includes Full workflow test, Catalog ready, JPEG under 500 KB, Square & centered and AI studio scene. Presets fill editable instructions only. Review plan calls the configured DeepSeek planner; the returned plan must be started separately, with an explicit AI approval button for Google Batch generation. Changing the instruction or selected image scope invalidates the displayed plan. The full test covers edits, centering, resizing, AI generation, JPEG conversion and target-size compression. Cloud end-to-end execution requires configured keys and has not been performed as part of adding these presets. Compression continues through the local Rust encoder and target-size search; the referenced webp2jpg-online browser encoder is not bundled.

## Natural scenario presets (2026-10-13)
The verbose test instructions above are superseded by concise Chinese user scenarios, starting with 产品主图，800*800，100kb. Assistant planning receives separate application defaults: WebP unless explicitly overridden, pixel dimensions, KB as 1000 bytes, and product-hero intent as AI minimalist still-life photography. These defaults do not implement a histogram-driven enhancement engine or semantic product-boundary crop. Both remain pending evaluation of mature image-processing projects; the planner must disclose these limits and never invent pixel analysis.


## Radial painting tools
Right-click the image canvas to open Color, Brush and Eraser. The translucent sector wheel follows the EDGE theme. Color contains 71 swatches and an outlined image-eyedropper cell. Selecting a color returns to painting; parameter menus stay open.

Brush size, hardness and opacity extend as parallel short arcs outside the selected tool icon, without a background panel. Red circle, diamond and half-circle handles distinguish the parameters. Drag a handle, scroll over its control, or focus it and use arrow keys. Click the retained tool icon to return to the wheel. Escape returns one menu level, then closes. Click Done to return to normal canvas interaction; hold Space to pan temporarily. The eraser affects paint only. Each stroke supports Undo/Redo, survives local draft recovery, and is included in saved versions and PNG/WebP/JPEG exports. Crops and transforms also transform the strokes; base color adjustments preserve the chosen paint color.

Painting regressions: `node --conditions=browser --test scripts/test-paint.mjs`; Rust rendering tests are included in `cargo test`. The implementation decisions are in `docs/decisions-radial-paint-menu.md`.

## Small-team LAN access
Keep using `一键启动.cmd` on the host. Settings > Team access creates colleague accounts. Run `开启局域网访问.cmd` once to allow HTTPS 8788 from the Private local subnet. Colleagues install the host-provided certificate and sign in to the displayed HTTPS address. Uploads, exports, events and projects are account-isolated; API calls reuse host credentials. See [TEAM-LAN.md](docs/TEAM-LAN.md) for certificate setup, privacy boundaries and queue behavior.

## 中文体验更新（2026-10-23）
界面已改为中文，按清晰技术写作原则统一操作名称。支持本机保存目录、Process 子目录下载、批量分段上传、逐图进度、处理页选图、项目删除撤销、拖动浏览、实时与异步模式、Poe GPT Image 2.5，以及同事问题反馈。见 [本轮说明](docs/UX-2026-10-23.md)。Poe 需要主机配置自己的 API 密钥；本轮未执行付费供应商调用。

## 同事图片临时处理
同事选择本机目录后自动保存，写入与校验成功后清理主机对应副本。未保存内容保留 24 小时；本机个人项目、账号和问题汇总不清理。普通下载需本人确认保存，不能仅凭浏览器开始下载就删除。详见 [临时处理规则](docs/TEMPORARY-WORKSPACES.md)。
