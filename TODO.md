# TODOs

There are several areas I would like to improve to allow `rx` to become a nice daily driver
for me in pixel-editing workflows:

- Key-Maps
  - [ ] I don't love not being able to set up key-maps with modifier keys, f.e. `shift + u` for `redo`
- More brush modes / settings:
  - Primitives
    - [x] Circle
    - [x] Rectangle
    - etc.
  - Brush heads, which are currently always a square of the set size
    - [ ] Circle head
    - [ ] Dither-head where the second color is used for the *off*-pattern
  - UI
    - [ ] Show which brush modes are currently active, maybe via icons somewhere
    - [ ] Allow clicking brush-mode icons to toggle. Those could be next to the colors
- Layers
  - This is kind of blocked by a custom file-format, as we would need to save that information somewhere
  - [ ] Expand frame system vertically, so we have horizontal == frames and vertical == layers, with the
    topmost layer being a *preview* of that frame with all layers applied
  - [ ] Allow reordering layers
  - [ ] Allow naming and renaming layers
  - [ ] Already think about how layers could support masks
- Preview
  - A designated window area that always shows the currently active canvas at a 100% zoom
  - [ ] Add a setting for preview and make that toggleable
  - [ ] Add setting for which window corner to place preview
- File-Format improvements
  - `.png` allows custom chunks if I remember correctly, so we could store metadata for `px` and read that out on start:
    - The number if frames, so we can `f/slize` automatically when loading a file
    - Layer information (if present)
    - Undo stack (maybe)
  - On second thought: A `.png` asset should be minimal in size to not bloat the software that uses its
    so we probably need a dedicated file-format for `px` and allow exporting `.png` files instead
    - [x] A nice format for export would be `.gif` since we deal in frames/animations already
    - [x] On export users should be able to select which frame(s) to export
- IO improvements
  - [ ] `.aseprite` is super popular for pixel art so doing interop with that would be quite nice
- Keyboard drawing
  - For a lot of *precise* edits I would much rather **not** use my mouse. The current design of `px` does not
    really cater to that workflow though. For this to become more practical, both `normal` and `visual` mode would
    need substsantial additions to make manipulating selections easier.
  - I think the current design with a brush is not the proper abstraction for that, rather we should think in **strokes**,
    where a stroke is performed between a *start* and an *end*. *Vim* only really deals in insertion/edit points so
    an approach to fix this for `px` needs to differ somehow.
  - It's a bit simpler for *visual* mode, where we manipulate a selection and then either fill it with a solid color,
    (or potentially in the future a dither pattern), or *yank*, *delete*, etc. it. Here an easy way to improve that workflow
    would be to implement more operators for selection manipulation f.e.
    - select/deselect next/prev row/col
    - selection repeats like `5n`
    - multi-selection support
    - selection primitives (rectangle / circle / polygon)
