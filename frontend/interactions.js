import { h, icon } from "./dom.js";
import { appStore } from "./state.js";

let scopedRouteGuardBound = false;

/**
 * Build a square icon-only button for dynamically inserted controls.
 */
function iconButton(label, iconName, className = "icon-button", attrs = {}) {
  return h("button", { class: className, type: "button", "aria-label": label, ...attrs }, [icon(iconName)]);
}

/**
 * Bind all route-local interactions after each render pass.
 */
export function bindInteractions(render, pluginInitials) {
  bindScopedRouteGuard();
  document.querySelectorAll("a[data-link]").forEach((anchor) => {
    anchor.addEventListener("click", (event) => {
      if (anchor.closest("[data-scoped-region]")) return;
      event.preventDefault();
      window.location.hash = anchor.dataset.link;
      render();
    });
  });
  bindPanels();
  bindComposer();
  bindMessage();
  bindTerminal();
  bindSettingsForms();
  bindDialogs(pluginInitials);
}

/**
 * Prevent accidental route anchors inside scoped regions from changing hash.
 */
function bindScopedRouteGuard() {
  if (scopedRouteGuardBound) return;
  scopedRouteGuardBound = true;
  document.addEventListener("click", (event) => {
    const anchor = event.target.closest?.("[data-scoped-region] a[data-link]");
    if (!anchor) return;
    event.preventDefault();
  }, true);
}

/**
 * Keep provider-specific fields visible only when the provider needs them.
 */
function bindSettingsForms() {
  const providerType = document.querySelector("#provider-type");
  if (!providerType) return;
  const alwaysVisible = new Set(["provider-type", "provider-label", "api-key"]);
  const hostedProviderTypes = new Set(["OpenAI", "OpenRouter", "Hugging Face router"]);
  const syncProviderFields = () => {
    // Hosted providers can fill URL/auth details later; custom endpoints need
    // the full field set available in the r04 order.
    const showAll = !hostedProviderTypes.has(providerType.value);
    document.querySelectorAll("[data-provider-field]").forEach((field) => {
      field.hidden = !showAll && !alwaysVisible.has(field.dataset.providerField);
    });
  };
  providerType.addEventListener("change", syncProviderFields);
  syncProviderFields();
}

/**
 * Bind shell collapse controls and side-panel drag handles.
 */
function bindPanels() {
  const shell = document.querySelector(".app-shell");
  if (!shell) return;
  shell.querySelectorAll("[data-panel-toggle]").forEach((control) => {
    control.addEventListener("click", () => {
      const side = control.dataset.panelToggle;
      const collapsed = !shell.classList.contains(`is-${side}-collapsed`);
      shell.classList.toggle(`is-${side}-collapsed`, collapsed);
      appStore.setShellValue(`${side}Collapsed`, collapsed);
      control.setAttribute("aria-pressed", String(collapsed));
    });
  });
  shell.querySelectorAll("[data-resize-panel]").forEach((handle) => {
    handle.addEventListener("pointerdown", (event) => {
      const side = handle.dataset.resizePanel;
      const prop = side === "left" ? "--left-panel" : "--right-panel";
      const panel = side === "left" ? shell.querySelector(".sidebar") : shell.querySelector(".tool-panel");
      const start = side === "left" ? event.clientX : -event.clientX;
      const startWidth = panel.getBoundingClientRect().width;
      handle.setPointerCapture(event.pointerId);
      const move = (moveEvent) => {
        const current = side === "left" ? moveEvent.clientX : -moveEvent.clientX;
        const { max, min } = panelWidthLimits(shell, side);
        // Clamp against live layout space so a wide right panel cannot push
        // itself outside the visible browser viewport.
        const width = Math.max(min, Math.min(max, startWidth + current - start));
        shell.style.setProperty(prop, `${Math.round(width)}px`);
        appStore.setShellValue(`${side}Width`, Math.round(width));
      };
      const up = () => {
        handle.removeEventListener("pointermove", move);
        handle.removeEventListener("pointerup", up);
      };
      handle.addEventListener("pointermove", move);
      handle.addEventListener("pointerup", up);
    });
  });
}

/**
 * Calculate safe side-panel width boundaries for the current viewport.
 */
function panelWidthLimits(shell, side) {
  const left = shell.querySelector(".sidebar")?.getBoundingClientRect().width || 0;
  const right = shell.querySelector(".tool-panel")?.getBoundingClientRect().width || 0;
  const handles = 14;
  const workbenchMin = 520;
  const otherPanel = side === "left" ? right : left;
  const available = window.innerWidth - otherPanel - handles - workbenchMin;
  const designMin = side === "left" ? 220 : 300;
  const designMax = side === "left" ? 460 : 560;
  const max = Math.max(designMin, Math.min(designMax, available));
  const min = Math.min(designMin, max);
  return { max, min };
}

/**
 * Bind composer attachments and composer popover selection.
 */
function bindComposer() {
  document.querySelectorAll(".composer").forEach((node) => {
    const surface = node.dataset.composerSurface || "default";
    const composer = appStore.composerFor(surface);
    const input = node.querySelector(".attachment-input");
    const preview = node.querySelector("[data-attachments]");
    let attachments = composer.attachments;
    node.querySelector(".prompt-box")?.addEventListener("input", (event) => {
      appStore.setComposerValue(surface, "prompt", event.currentTarget.textContent || "");
    });
    node.querySelector("[data-attach-button]")?.addEventListener("click", () => input.click());
    input?.addEventListener("change", () => {
      attachments = attachments.concat(Array.from(input.files || []));
      appStore.setComposerValue(surface, "attachments", attachments);
      input.value = "";
      renderAttachments();
    });

    const renderAttachments = () => {
      preview.hidden = attachments.length === 0;
      preview.replaceChildren(...attachments.map((file, index) => h("span", { class: "attachment-chip" }, [
        icon("file"),
        h("span", { text: file.name }),
        h("small", { text: `${file.size} B` }),
        h("button", {
          class: "attachment-remove",
          type: "button",
          "aria-label": `Remove attachment: ${file.name}`,
          "data-remove-attachment": String(index)
        }, ["x"])
      ])));
      preview.querySelectorAll("[data-remove-attachment]").forEach((remove) => {
        remove.addEventListener("click", () => {
          attachments.splice(Number(remove.dataset.removeAttachment), 1);
          appStore.setComposerValue(surface, "attachments", attachments);
          renderAttachments();
        });
      });
    };
    node.querySelectorAll("[data-popover-trigger]").forEach((trigger) => {
      trigger.addEventListener("click", () => {
        const popover = node.querySelector(`[data-popover='${trigger.dataset.popoverTrigger}']`);
        popover.hidden = !popover.hidden;
      });
    });
    node.querySelectorAll("[data-popover-option]").forEach((option) => {
      option.addEventListener("click", () => {
        const kind = option.closest("[data-popover]").dataset.popover;
        const control = node.querySelector(`[data-popover-trigger='${kind}']`);
        const trigger = control?.querySelector("span");
        if (trigger && option.textContent !== "+ Create branch") {
          trigger.textContent = option.textContent;
          appStore.setComposerValue(surface, kind, option.textContent);
          if (kind === "approval") control.setAttribute("aria-label", `Approval policy: ${option.textContent}`);
          if (kind === "branch") control.setAttribute("aria-label", `Branch: ${option.textContent}`);
        }
        option.closest("[data-popover]").hidden = true;
      });
    });
    renderAttachments();
  });
}

/**
 * Bind the chat message disclosure control.
 */
export function bindMessage() {
  document.querySelector("[data-toggle='message']")?.addEventListener("click", (event) => {
    const message = event.currentTarget.closest(".message");
    const expanded = message.classList.toggle("is-expanded");
    event.currentTarget.textContent = expanded ? "Show Less" : "Show More";
    event.currentTarget.setAttribute("aria-expanded", String(expanded));
  });
}

/**
 * Bind the vertical resize handle inside the Terminal tool panel.
 */
export function bindTerminal() {
  const handle = document.querySelector("[data-resize-stack='terminal']");
  if (!handle) return;
  handle.addEventListener("pointerdown", (event) => {
    const panel = handle.closest(".terminal-tool");
    const bottom = panel.querySelector(".terminal-bottom");
    const start = event.clientY;
    const startHeight = bottom.getBoundingClientRect().height;
    handle.setPointerCapture(event.pointerId);
    const move = (moveEvent) => {
      const height = Math.max(120, Math.min(420, startHeight + start - moveEvent.clientY));
      panel.style.setProperty("--terminal-bottom", `${Math.round(height)}px`);
      appStore.setShellValue("terminalBottom", Math.round(height));
    };
    const up = () => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", up);
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", up);
  });
}

/**
 * Bind plugin, marketplace, skill, and MCP dialog interactions.
 */
function bindDialogs(pluginInitials) {
  const openModal = (modal, trigger) => {
    modal.hidden = false;
    const title = modal.querySelector("[data-plugin-modal-title]");
    if (title && trigger?.dataset.pluginConnect) title.textContent = `Connect ${trigger.dataset.pluginConnect}`;
    const target = modal.querySelector("[data-plugin-modal-target]");
    if (target && trigger?.dataset.pluginConnect) target.textContent = pluginInitials(trigger.dataset.pluginConnect);
    const continueLabel = modal.querySelector("[data-plugin-continue-label]");
    if (continueLabel && trigger?.dataset.pluginConnect) continueLabel.textContent = `Continue to ${trigger.dataset.pluginConnect}`;
    modal.querySelector("button")?.focus();
  };
  const close = (control) => control.closest(".plugin-modal").hidden = true;
  document.querySelectorAll("[data-plugin-connect]").forEach((trigger) => trigger.addEventListener("click", () => openModal(document.querySelector("[data-plugin-modal='connect']"), trigger)));
  document.querySelector("[data-marketplace-menu-trigger]")?.addEventListener("click", () => {
    const trigger = document.querySelector("[data-marketplace-menu-trigger]");
    const menu = document.querySelector("[data-marketplace-menu]");
    menu.hidden = !menu.hidden;
    trigger.setAttribute("aria-expanded", String(!menu.hidden));
  });
  document.querySelectorAll(".marketplace-option").forEach((option) => {
    option.addEventListener("click", () => {
      const trigger = document.querySelector("[data-marketplace-menu-trigger]");
      const menu = document.querySelector("[data-marketplace-menu]");
      menu.hidden = true;
      trigger.setAttribute("aria-expanded", "false");
    });
  });
  document.querySelector("[data-marketplace-open]")?.addEventListener("click", () => {
    const trigger = document.querySelector("[data-marketplace-menu-trigger]");
    const menu = document.querySelector("[data-marketplace-menu]");
    if (menu) menu.hidden = true;
    trigger?.setAttribute("aria-expanded", "false");
    openModal(document.querySelector("[data-marketplace-modal]"));
  });
  document.querySelectorAll("[data-skill-open]").forEach((trigger) => trigger.addEventListener("click", () => {
    const modal = document.querySelector("[data-skill-modal]");
    modal.querySelector("[data-skill-title]").textContent = trigger.dataset.skillOpen;
    openModal(modal, trigger);
  }));
  document.querySelector("[data-mcp-add]")?.addEventListener("click", () => openModal(document.querySelector("[data-mcp-modal]")));
  document.querySelectorAll("[data-mcp-mode]").forEach((control) => control.addEventListener("click", () => {
    const modal = control.closest("[data-mcp-modal]");
    modal.querySelectorAll("[data-mcp-mode]").forEach((mode) => {
      const active = mode === control;
      mode.classList.toggle("is-active", active);
      mode.setAttribute("aria-selected", String(active));
    });
    modal.querySelectorAll("[data-mcp-fields]").forEach((fields) => fields.hidden = fields.dataset.mcpFields !== control.dataset.mcpMode);
  }));
  document.querySelectorAll(".mcp-add-line").forEach((control) => {
    control.addEventListener("click", () => addMcpGroupRow(control.closest(".mcp-group")));
  });
  document.querySelectorAll("[data-mcp-remove-row]").forEach((control) => {
    control.addEventListener("click", () => removeMcpGroupRow(control.closest(".mcp-group"), control.closest("[data-mcp-row]")));
  });
  document.querySelectorAll("[data-plugin-close], [data-marketplace-close], [data-skill-close], [data-mcp-close]").forEach((control) => control.addEventListener("click", () => close(control)));
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    document.querySelectorAll(".plugin-modal:not([hidden])").forEach((modal) => modal.hidden = true);
  }, { once: true });
}

/**
 * Add another row to a dynamic MCP field group.
 */
function addMcpGroupRow(group) {
  if (!group) return;
  const title = group.dataset.mcpGroup;
  const rowClass = group.dataset.mcpColumns === "pair" ? "mcp-pair-row" : "mcp-single-row";
  const placeholders = group.dataset.mcpColumns === "pair" ? ["Key", "Value"] : [""];
  const index = group.querySelectorAll("[data-mcp-row]").length + 1;
  const row = h("div", { class: rowClass, "data-mcp-row": String(index) }, [
    ...placeholders.map((placeholder, inputIndex) => h("input", {
      "aria-label": `${title} ${index}.${inputIndex + 1}`,
      placeholder,
      spellcheck: "false",
      type: "text"
    })),
    iconButton(`Remove ${title} row ${index}`, "trash", "mcp-trash", { "data-mcp-remove-row": true })
  ]);
  row.querySelector("[data-mcp-remove-row]").addEventListener("click", (event) => {
    removeMcpGroupRow(group, event.currentTarget.closest("[data-mcp-row]"));
  });
  group.insertBefore(row, group.querySelector(".mcp-add-line"));
}

/**
 * Remove an MCP field-group row while preserving one editable row.
 */
function removeMcpGroupRow(group, row) {
  if (!group || !row || group.querySelectorAll("[data-mcp-row]").length <= 1) return;
  row.remove();
  group.querySelectorAll("[data-mcp-row]").forEach((currentRow, index) => {
    const rowNumber = index + 1;
    currentRow.dataset.mcpRow = String(rowNumber);
    currentRow.querySelector("[data-mcp-remove-row]")?.setAttribute("aria-label", `Remove ${group.dataset.mcpGroup} row ${rowNumber}`);
  });
}
