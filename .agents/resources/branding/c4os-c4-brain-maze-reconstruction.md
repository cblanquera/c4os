# C4OS C4 Brain Maze Reconstruction Guide

## Purpose

This file is a standalone construction specification for the bottom-left mark
from `c4os-brain-maze-concept-board-v1.png`.

The target mark combines three readings in one symbol:

1. a frontal human brain;
2. a short, thick maze; and
3. the monogram `C4`.

The instructions and SVG data below are sufficient to recreate the mark
without the concept-board PNG. The SVG is a measured, regularized vector
reconstruction. It preserves the selected idea and proportions while removing
raster antialiasing, slight shading, and uneven placement introduced by image
generation.

This is a reconstruction guide, not a declaration that the mark is the final
installed C4OS logo. Do not overwrite application icons until the vector has
been visually accepted.

## Required Visual Result

- The first read must be a symmetrical, frontal brain.
- The second read must be the characters `C4` integrated into maze corridors.
- The `C` occupies the left hemisphere and opens toward the center.
- The `4` occupies the right hemisphere and shares the brain's corridor logic.
- The two hemispheres remain visibly separated at the top and bottom.
- The mark uses one flat dark color. There are no gradients, shadows, textures,
  outlines of a second color, or anatomical details.
- All terminals and corners are rounded and share one geometric language.
- The symbol must remain recognizable without a wordmark.

## Source Measurements

These measurements record provenance; they are not needed when using the SVG
recipe below.

- Source board: `1536 × 1024` pixels.
- Selected board cell: bottom-left, nominal crop `x=0–511`, `y=512–1023`.
- Selected-cell size: `512 × 512` pixels.
- Dark-pixel bounds inside that cell, using luminance `< 80`:
  `x=98–482`, `y=64–396`.
- Approximate source mark bounds: `385 × 333` pixels.
- Source-sampled dark: `#191B18`.
- Source-sampled cream: `#F7F5F0`.
- The source PNG has no alpha channel.

The generated board placed the mark slightly right of the cell center. The
vector recipe intentionally centers the symbol optically instead of preserving
that accidental board placement.

## Coordinate System

Use a square SVG artboard:

```text
viewBox:        0 0 100 100
stroke width:   8 units
line cap:       round
line join:      round
fill:           none
master color:   currentColor
preview color:  #191B18
preview field:  #F7F5F0
```

The mark's approximate live area is:

```text
left:    8
right:   99
top:     11
bottom:  92
```

Do not move individual paths after construction. Scale the complete group
uniformly if a different live area is needed. Never stretch it horizontally or
vertically.

## Structural Measurements

| Feature | Measurement |
| --- | --- |
| Base stroke | `8` units |
| Implied corner radius | `4` units from round joins/caps |
| Left central spine | `x=49` |
| Right central stem | `x=62` |
| Upper hemisphere starts | `(49,15)` and `(61,15)` |
| Visible top-center gap | `4` units after stroke thickness |
| Lower stems end | `y=88` |
| Visible bottom-center gap | `5` units after stroke thickness |
| C upper turn | `(49,39)` |
| C left reach | `x=34` |
| C lower turn | `(34,51)` |
| 4 diagonal | `(80,34)` to `(59,55)` |
| 4 crossbar | `y=55`, from `x=59` to `x=88` |
| 4 upright | `x=80`, from `y=43` to `y=61` |

The `4` diagonal is exactly a 45-degree descending line. The C uses only
orthogonal segments. This contrast is intentional: the left side feels like a
maze, while the diagonal makes the `4` immediately legible.

## Path Inventory

Create the paths in this order. Every path uses the same stroke settings.

### 1. Left brain contour

```text
M49 15
H41
C34 15 29 20 29 28
H25
C18 28 14 34 15 41
C12 46 12 54 15 59
C18 64 22 66 28 66
C28 74 34 80 43 80
H49
```

This open path creates the outer left hemisphere. Its upper and lower ends
align with the left central spine.

### 2. C corridor and left central spine

```text
M49 15
V39
H34
V51
H49
V88
```

The upper vertical segment forms the brain's center cleft. The left-right jog
forms the C-shaped maze route. The final vertical segment becomes the lower
center stem.

### 3. Right brain contour

```text
M61 15
H69
C76 15 81 20 81 28
H84
C90 28 94 34 93 41
C95 46 95 54 93 59
C90 64 86 66 81 66
C81 74 75 80 68 80
```

This is intentionally similar to, but not a mathematical mirror of, the left
contour. The right contour leaves room for the 4's diagonal and crossbar.

### 4. Main 4 diagonal and crossbar

```text
M80 34
L59 55
H88
```

This single path provides the 4's diagonal and horizontal crossbar. Its right
terminal visually joins the right brain lobe.

### 5. Short 4 upright

```text
M80 43
V61
```

This segment must overlap the diagonal/crossbar path cleanly. Do not leave a
cream seam between them.

### 6. Right lower central stem

```text
M62 59
V88
```

This completes the lower brain cleft and balances the long lower stem on the
left.

## Complete Reference SVG

Copy this code exactly to recreate the cream-background reference image:

```svg
<svg xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 100 100"
  role="img"
  aria-label="C4 brain maze logo">
  <rect width="100" height="100" fill="#F7F5F0" />
  <g
    fill="none"
    stroke="#191B18"
    stroke-width="8"
    stroke-linecap="round"
    stroke-linejoin="round">
    <path d="M49 15H41C34 15 29 20 29 28H25C18 28 14 34 15 41C12 46 12 54 15 59C18 64 22 66 28 66C28 74 34 80 43 80H49" />
    <path d="M49 15V39H34V51H49V88" />
    <path d="M61 15H69C76 15 81 20 81 28H84C90 28 94 34 93 41C95 46 95 54 93 59C90 64 86 66 81 66C81 74 75 80 68 80" />
    <path d="M80 34L59 55H88" />
    <path d="M80 43V61" />
    <path d="M62 59V88" />
  </g>
</svg>
```

## Transparent Master SVG

For a reusable one-color master:

1. remove the `<rect>` element;
2. change the group's stroke from `#191B18` to `currentColor`; and
3. apply the desired color from the consuming HTML, CSS, or design system.

The opening should become:

```svg
<svg xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 100 100"
  role="img"
  aria-label="C4 brain maze logo">
  <g
    fill="none"
    stroke="currentColor"
    stroke-width="8"
    stroke-linecap="round"
    stroke-linejoin="round">
```

Do not use CSS inversion for a production colored logo. Set `color` explicitly
for each surface.

## Manual Reconstruction in a Vector Editor

Use these steps in Figma, Illustrator, Affinity Designer, Sketch, or Inkscape:

1. Create a `100 × 100` frame or artboard.
2. Turn on a one-unit grid and snap to whole coordinates.
3. Set the drawing tool to an `8`-unit stroke, no fill, round caps, and round
   joins.
4. Draw the left brain contour using the coordinates in Path 1.
5. Draw the C corridor using Path 2. Confirm its endpoints exactly overlap the
   left contour at `(49,15)` and visually align at the lower center.
6. Draw the right brain contour using Path 3.
7. Draw the 4 diagonal and crossbar as one path using Path 4.
8. Add the short upright from Path 5 and the lower center stem from Path 6.
9. Set every path to the same dark color. There must be no lighter path,
   outline, inner shadow, or transparency difference.
10. Inspect all overlaps at 800% zoom. Same-color strokes must merge into a
    single uninterrupted silhouette with no cream pinholes.
11. Inspect the mark at normal size and at `16 × 16`. The brain and `C4` must
    both remain recognizable.
12. Keep editable strokes in the working file. Convert strokes to outlines only
    in a duplicate intended for final distribution.

Do not substitute a font for the C or 4. Their proportions come from the maze
geometry, not a typeface.

## Optical Rules

- Preserve the top and bottom clefts. Joining either pair of stems turns the
  brain into a closed badge and weakens the two-hemisphere reading.
- Keep the left C orthogonal. Rounding its path into a typographic C weakens the
  maze character.
- Keep the 4 diagonal at 45 degrees. A shallower diagonal makes the right side
  look like an arrow or generic route.
- The 4 crossbar may merge into the outer lobe, but it must not extend beyond
  the brain silhouette.
- The right side is optically denser because the 4 has three strokes. Do not
  compensate by thinning it; the shared `8`-unit stroke is part of the mark's
  coherence.
- The outer brain should feel rounded, but not anatomical. Use only the broad
  lobe curves described by the paths.

## Allowed Tolerances

When adapting the mark manually:

- stroke width: `7.5–8.5` units;
- path coordinate adjustment: no more than `1` unit;
- outer-lobe curve adjustment: no more than `2` units;
- top and bottom center gaps: never less than `3` units after stroke thickness;
- diagonal angle: `45° ± 2°`;
- uniform scaling: allowed;
- non-uniform scaling, rotation, skew, and perspective: not allowed.

If a change exceeds these tolerances, treat it as a new logo variation rather
than a faithful reconstruction.

## Export Guidance

Use SVG as the editable master. Recommended raster exports:

| Use | Size | Background |
| --- | --- | --- |
| App/marketing master | `1024 × 1024` | transparent or approved field |
| Social avatar | `512 × 512` | approved field |
| General UI | `64 × 64` | transparent |
| Small UI | `32 × 32` | transparent |
| Favicon check | `16 × 16` | approved field |

Rasterize with antialiasing enabled. Do not add sharpening, blur, texture, or a
drop shadow after export.

For an app icon, place the mark on a separate approved container; do not bake a
rounded-square container into the core symbol.

## Prohibited Changes

- no gradients, glows, shadows, bevels, or 3D treatment;
- no realistic folds, neurons, circuits, nodes, or anatomical brain details;
- no arrowheads on maze terminals;
- no standalone `C4` text laid over a separate brain icon;
- no profile-head silhouette;
- no enclosing circle, shield, hexagon, or rounded app frame in the master;
- no different stroke widths between the brain, C, and 4;
- no square caps or sharp joins;
- no clipping at the SVG viewBox boundary;
- no external font or raster dependency.

## Reconstruction Acceptance Checklist

A reconstruction passes when all statements below are true:

- [ ] It renders from this file's SVG without external assets.
- [ ] It reads as a frontal brain before any explanation.
- [ ] The left side reads as `C` and the right side reads as `4`.
- [ ] The monogram also reads as a short maze route.
- [ ] Every visible stroke has equal weight.
- [ ] All corners and endpoints are rounded.
- [ ] The upper and lower center gaps remain open.
- [ ] No part of the stroke clips at the viewBox edge.
- [ ] It works as a one-color silhouette.
- [ ] It remains recognizable at `32 × 32`.
- [ ] At `16 × 16`, the brain silhouette and `C4` remain distinguishable even
      if the maze detail becomes secondary.
- [ ] The transparent master contains no background rectangle.
- [ ] The cream-preview version uses `#F7F5F0` and the dark mark uses
      `#191B18`.

## Final Production Note

This recipe establishes a reproducible geometry for the selected concept. It
does not prove trademark availability or originality, and it should receive a
final human visual review before replacing the current C4OS app icon.
