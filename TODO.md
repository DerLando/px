# TODOs

There are several areas I would like to improve to allow `rx` to become a nice daily driver
for me in pixel-editing workflows:

- Key-Maps: I don't love not being able to set up key-maps with modifier keys, f.e. `shift + u` for `redo`
- More brush modes / settings:
  - Primitives
    - Circle
    - Rectangle
    - etc.
  - Brush heads, which are currently always a square of the set size
    - Circle head
    - Dither-head where the second color is used for the *off*-pattern
- Layers
  - No idea how that would work UI wise though
- Preview
  - A designated window area that always shows the currently active canvas at a 100% zoom
- File-Format improvements
  - `.png` allows custom chunks if I remember correctly, so we could store metadata for `px` and read that out on start:
    - The number if frames, so we can `f/slize` automatically when loading a file
    - Layer information (if present)
    - Undo stack (maybe)
