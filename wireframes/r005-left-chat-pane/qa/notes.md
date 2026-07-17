# Visual QA — r005 Round 1

- Viewport reviewed: 1280 × 720 desktop.
- Initial geometry: left panel 228px; center begins at the same 228px boundary.
- Repeated width resize:
  - Drag 1: panel 228px → 380px; center width 900px.
  - Drag 2: panel 380px → 280px; center width 1000px.
  - Responsive maximum: panel clamped at 704px (55%); center retained 576px.
  - Return drag: panel returned to 300px; center returned to 980px.
  - Resize state cleared after each drag and the handle remained aligned at panel width minus its overlap.
- Chat pane:
  - Appeared only after Browser focus.
  - Defaulted to 288px, exactly 40% of the 720px viewport.
  - Resized to the 432px/60% maximum, then back to 340px across a second drag.
  - Retained the 340px height after Chat restoration and another artifact focus.
- Collapsed behavior:
  - Collapsing the left panel while Browser remained focused expanded center to the full 1280px.
  - Expanding Browser from Chat while the panel was already collapsed left the panel collapsed.
  - Reopening the panel revealed the retained Chat pane and Restore Chat control.
- Regression checks: composer stayed fixed, Terminal had zero Expand actions, no floating frame remained, no horizontal document overflow, and no console errors were reported.

## Captures

- `round-1-left-chat-pane.png`
- `round-1-left-chat-pane-resized.png`
- `round-1-collapsed-focus.png`

# Visual QA — r005 Round 2

- Viewport reviewed: 1119 × 911 desktop.
- Browser focus:
  - Inline Browser navigation controls visible in the left Chat pane: 0.
  - Read-only inline address remained visible and matched the focused address.
  - Composer mode chooser visible during focus: 0; composer was locked to Chat.
  - Focused Browser retained back, forward, refresh, address, and direct Close controls.
  - Closing from the center while the left panel was collapsed restored the centered thread and left the panel collapsed.
  - The previously selected Browser composer mode returned after restoration.
- File focus:
  - Inline File Edit, Save, Cancel, and approval controls visible in the left Chat pane: 0.
  - Focused File Edit used a pencil icon and exposed Cancel/Save only after activation.
  - Editor bottom, focus-viewer bottom, and composer top all aligned at 755px; the editor used the complete 641px available content height.
  - Direct Close restored the centered thread while preserving the collapsed left-panel state.
- Regression checks: Terminal still had zero Expand actions, the document had no horizontal overflow, and no runtime errors were observed.

## Captures

- `round-2-browser-focus.png`
- `round-2-file-focus.png`
