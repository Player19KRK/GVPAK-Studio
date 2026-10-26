# EDGE visual system
White and cool grey surfaces with signal red accents. Barlow Condensed wordmark/headings, compact neutral sans controls. Preserve the existing desktop editor language. Purposeful controls and brief English labels.

The central workspace is a 2D image editor. No Three.js stage, city, camera orbit or tour. Coverflow uses the original GVPAK STUDIO 2 perspective mathematics and motion, with EDGE colors and controls. It is image navigation, not a separate 3D viewing mode.

Persistent header: project, Edit/Generate/Export/Settings, Process, Save and Export. Tool rail on the left, canvas and Coverflow centrally, collapsible inspector on the right. Workflow screens use the same type, spacing and control vocabulary. Narrow screens collapse inspector and wrap mode navigation. Respect reduced motion.

Inspector behavior: left Adjust/Crop/Transform buttons toggle their own visible parameter panel. A different tool or a different inspector tab opens the requested tool parameters. Keyboard selection opens the panel without toggling. The general inspector toggle preserves the Layers/History tab. Closing preserves edits and returns focus to the tool. Disclosure controls expose expanded state; non-editor screens disable the inspector toggle. Use a brief 180 ms width transition, disabled for reduced motion.

Task scope must be explicit before batch export or processing. Search selection affects visible results only. Current-image Save is separate from batch scope. Preserve image drafts on reload and workflow form inputs when switching pages. Protect dirty drafts before restoring/adopting versions; give reversible trash actions. Validate export fields inline and report successful, unmet and cancelled items separately.

Project hover previews use a borderless, translucent surface with three columns and at most three visible rows. Use top-layer popovers so grid clipping never hides them. Prefer beside-card placement, flip at viewport edges, clamp on small screens. A short hover delay and pointer bridge keep previews stable. Preview wheel scrolling is contained. Project/image thumbnail zoom is separate from page or canvas zoom.

Current brand: GVPAK STUDIO, text only, no lightning or graphic mark. Desktop header 48px, canvas controls 38px/30px, status 22px. Default inspector 280px and Coverflow 144px; both user-resizable and locally persisted. Pointer capture, keyboard separators and numeric settings must share sizing state. Preserve physical image scale during panel resizing when manually zoomed. Focus hides panels temporarily without losing their size/visibility preference.

## Responsive image preview
The visible image has priority: show its available thumbnail/cache immediately, load a 1600px preview source after navigation settles, and use WebGL2 for interactive color, crop and transform feedback. Rust replaces the interactive result after a 220ms pause and remains authoritative for saved/exported pixels. Neutral edits reuse the source without a second render. Comparison originals load only when Compare opens; histograms run after the image in an idle callback. Queued obsolete previews are discarded. Already-running native work finishes and may populate the cache.

The LRU targets 12 entries, 32 MiB encoded blobs and 128 MiB estimated decoded pixels; displayed entries are pinned. These are cache targets, not a cap on total browser/GPU memory. Settings > Preview acceleration offers CPU compatibility; unavailable GPU contexts fall back automatically. GPU resizing is an interactive approximation, especially after crops; the settled Rust render provides precise output.

Double-click the image canvas to restore centered fit at the app's 100% (not native 1:1 pixels). Holding Space over Coverflow displays Select or Deselect near its page counter, then fades after release. The intent stays fixed until release.

Verification: `node --test scripts/test-preview-cache.mjs scripts/test-coverflow-selection.mjs`; GPU/Rust fixtures in `docs/verification/gpu-qa`; measured results in `docs/verification/gpu-preview-report.json`.

Image grid: click an image to toggle selection; double-click to edit. Keyboard Space toggles selection and Enter opens the editor. Project peek dismisses immediately when the pointer leaves its card, panel and narrow connecting gap; there is no close timer.


## Radial painting menu
Right-click only the image canvas to open a translucent EDGE wheel with three continuous sectors: Color, Brush, Eraser. Entering a child hides root sectors. Center returns one level; Escape returns one level, then closes. Clicking outside consumes the click. Position within the canvas when it fits, otherwise within the viewport. Preserve the image itself without blur.

Color uses three rings of 24 wedge cells, including one outlined eyedropper cell. Swatches are opaque. Choosing a color closes the menu and activates painting. Brush size, hardness and opacity extend as parallel short arcs outside the selected tool icon. Parameter mode has no background disk, outline ring or center button; clicking the retained tool icon returns to the root. Red handles use a circle for size, a diamond for hardness and a half-filled circle for opacity; wheel over a parameter adjusts only that parameter. Parameter menus stay open. Eraser exposes size and hardness and affects paint only. Honor reduced motion and keyboard focus.

Paint strokes are editable source-coordinate data. Each stroke is one undo step. They follow crop/rotate/flip and are included in drafts, saved versions, thumbnails and PNG/WebP/JPEG exports. Base color adjustments do not recolor paint. Hold Space to pan temporarily; Done returns to normal canvas interaction. The image eyedropper samples the precise edited composite, excluding UI and canvas background.


Radial menu sizing: the root footprint is 224 CSS px; palette is at most 288 CSS px and scales to the image canvas. Parameter layouts use their actual compact footprint, not the palette's square. Never fall back to window bounds merely because the canvas is below 300px high. Root placement reserves horizontal room for both parameter fans; parameter fans stay within the root's vertical extent so the retained tool icon does not jump on entering a submenu. Recompute on viewport and canvas resize. Use existing 10px control text, 18px Lucide icons, grey-green neutrals, 1px tracks and small signal-red handles; no extra title/footer or decorative perimeter rings.
