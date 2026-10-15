# GVPAK STUDIO

Svelte 5 image workspace, using the Rust image-processing, project, export and AI-task core migrated from GVPAK STUDIO 2.

## Start
Double-click `start.cmd`, then open http://127.0.0.1:8787/.
The bundled Windows x64 Rust executable serves the compiled interface and API. No Node/Rust installation is needed to run it. Denoise, automatic centering and deskew use an existing Python/OpenCV environment; Settings reports availability.

The Three.js spatial preview has been removed. Coverflow follows the original GVPAK STUDIO 2 motion and selection behavior, styled to match EDGE.

## Workflows
- Edit: local imports, color, free/ratio crop, transforms, comparison, zoom/pan, layers, RGB histogram and saved versions.
- Projects: click EDGE or the project name. Create or rename projects, scroll stacked project covers, browse/multiselect images, trash/restore.
- Generate: configure Google in Settings, enter a prompt, approve the individual request, then review/adopt a result.
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
