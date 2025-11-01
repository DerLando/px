# TODOs

There are several areas I would like to improve to allow `rx` to become a nice daily driver
for me in pixel-editing workflows:

- Key-Maps
  - [ ] I don't love not being able to set up key-maps with modifier keys, f.e. `shift + u` for `redo`
- More brush modes / settings:
  - Primitives
    - [x] Circle
    - [ ] Rectangle
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
    - [ ] A nice format for export would be `.gif` since we deal in frames/animations already
    - [ ] On export users should be able to select which frame(s) to export
