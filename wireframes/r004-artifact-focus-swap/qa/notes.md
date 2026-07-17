# Visual QA — r004 Round 1

- Viewport reviewed: 1280 × 720 desktop.
- Browser focus: full-center Browser controls and page surface render behind the floating Chat frame; composer remains anchored and usable.
- File focus: full-center File surface supports Edit and Save; saving advances the version and updates the original bubble state.
- Frame behavior: moved from its default position, resized from 380 × 420 to the 320px minimum width, and retained the resulting 320 × 365 geometry across File/Browser focus changes and Chat restoration.
- Restoration: Chat returns to its original center host; the focused artifact placeholder is removed and its complete bubble is restored.
- Terminal: initial and generated Terminal cards contain Copy/Reply only and no Expand action.
- Layout: no right panel, right separator, tabs, horizontal page overflow, or console errors.

## Captures

- `round-1-browser-focus.png`
- `round-1-file-focus.png`
- `round-1-chat-restored.png`
