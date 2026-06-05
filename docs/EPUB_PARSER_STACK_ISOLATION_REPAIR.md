# EPUB parser stack isolation repair

Marker: `rustmix-wave=reader-epub-parser-stack-isolation-ready`

## Observed device failure

After the XML attribute tokenizer repair, physical firmware reached the first real EPUB archive stage and then reset. The monitor sequence was:

1. `screen-route route=reader-loading`
2. `reader-cache-stage route=reader-loading stage=Inspecting EPUB archive`
3. `***ERROR*** A stack overflow in task main has been detected.`

The earlier host fixture did not expose this target-only stack budget issue because the embedded firmware main task carries the product shell, hardware services and redraw loop in addition to the EPUB parser call chain.

## Repair

`ReaderUiState::tick()` now calls `open_epub_on_worker()` at the existing `InspectingEpubArchive` stage. The new boundary creates a short-lived `epub-parser` worker with an explicit 64 KB stack, runs bounded archive parsing, DEFLATE expansion and XHTML flattening there, then synchronously joins and hands the heap-backed `EpubDocument` back to the existing staged Reader flow.

The accepted firmware main-task stack remains 16 KB. This avoids expanding the long-lived main task merely to cover a bursty parser workload.

## Guardrails

- The underlying `open_epub()` parser remains bounded and is marked `#[inline(never)]` so the dedicated stack boundary remains explicit.
- Worker start failures and worker panics return Reader loading errors instead of silently continuing.
- Runtime markers report parser-worker start, completion, bounded parse failure or worker-start failure for physical smoke-test diagnosis.
- Host coverage opens the synthetic EPUB directly and through the dedicated worker, then compares the resulting document.
- Repository-contract validation checks the worker API, 64 KB stack budget, Reader call route, firmware marker, host test and preserved 16 KB main-task configuration.

## Preserved baseline

The repair does not change TXT pagination or normalization, `POSITS.TXT`, eight-character cache filenames, bookmark byte-offset authority, bookmark page labels, Reader Preferences, paragraph alignment, High Contrast rendering, Power-key sleep images, network suspension, EPUB TOC parsing or XML attribute handling.
