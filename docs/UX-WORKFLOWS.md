# Workflow UX improvements — 2026-10-13

## User choices
- Keep local editing drafts automatically; Save creates an explicit version.
- Default batch scope to checked images, or the current image when no selection exists.

## Changes
1. Current-image Save and Ctrl+S are independent of batch selection; clean images disable Save.
2. Image edits persist locally, keyed by data directory and asset. Matching saved-base fingerprints restore drafts and last active image after reload. Drafts never store API keys or pixels. A changed saved base prevents replaying obsolete edits.
3. Revert draft returns to the saved edit through undoable editing state.
4. Export/Process show a shared Current/Selected/All-project scope control and thumbnails. Trashed images are excluded. Single-image generation states its scope and invalidates paid approval when the reference or request changes.
5. Import enters the canvas and selects the first newly imported asset instead of an old image. Empty projects expose actionable import controls.
6. Search bulk selection affects visible results. Empty searches offer Clear search. Project rename has a visible button. Trashed images cannot be opened for editing; each has Restore.
7. Moving images to trash offers an Undo action. Version restore/adoption offers Save version first, Replace draft, or Cancel when edits would be replaced.
8. Generate/Export/Process retain their form state when switching pages. Settings opened from connection prompts has a return action. Only image editing drafts persist across browser reload; workflow form persistence is within the running page.
9. Export validates output dimensions, pixel count, quality, file limit and destination before submission. Optional file limit; PNG hides inapplicable quality. Results distinguish successes, failures/size targets and unprocessed cancellations, with copyable paths.
10. Export preview and local sample source agree with batch scope. Right-side Coverflow selection checkboxes move outward to avoid central-card occlusion; original motion geometry remains unchanged.
11. Task notices open the task list. Exact numeric inputs update on input. Keyboard tool cycling follows the three parameter tools and respects busy/text-entry states. Hold-original restores the previous compare state on release.

## Validation
Production/component checks plus live browser workflow checks. No cloud generation or paid planning requests executed. Backend processing semantics are unchanged.
