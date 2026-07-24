import type { ReactNode } from "react";
import type { Key } from "react-aria-components";
import {
  Dialog,
  DialogTrigger,
  Heading,
  Menu,
  MenuItem,
  MenuTrigger,
  Modal,
  ModalOverlay,
  Popover,
} from "react-aria-components";

import { Button } from "./controls";
import { Icon } from "./icons";

export interface ActionMenuItem {
  readonly description?: string;
  readonly id: Key;
  readonly isDisabled?: boolean;
  readonly label: string;
  readonly onAction: () => void;
  readonly shortcut?: string;
}

export interface ActionMenuProps {
  readonly align?: "start" | "center" | "end";
  readonly items: readonly ActionMenuItem[];
  readonly label: string;
  readonly triggerIcon?: ReactNode;
  readonly triggerText?: string;
}

export interface PopoverDialogProps {
  readonly children: ReactNode;
  readonly placement?: "top" | "bottom" | "left" | "right";
  readonly title: string;
  readonly triggerIcon?: ReactNode;
  readonly triggerLabel: string;
  readonly triggerText?: string;
}

export interface ModalDialogProps {
  readonly children: ReactNode;
  readonly closeLabel?: string;
  readonly isDismissable?: boolean;
  readonly renderActions?: (close: () => void) => ReactNode;
  readonly title: string;
  readonly triggerLabel: string;
  readonly triggerVariant?: "primary" | "secondary" | "quiet" | "danger";
}

export interface ControlledModalDialogProps {
  readonly children: ReactNode;
  readonly closeLabel?: string;
  readonly isDismissable?: boolean;
  readonly isOpen: boolean;
  readonly onDismiss: () => void;
  readonly renderActions?: () => ReactNode;
  readonly title: string;
}

/** Renders a focus-managed action menu that restores its trigger on dismiss. */
export function ActionMenu({
  align = "end",
  items,
  label,
  triggerIcon,
  triggerText,
}: ActionMenuProps) {
  return (
    <MenuTrigger>
      <Button
        {...(triggerText
          ? {}
          : { "aria-label": label, className: "c4-icon-button" })}
        variant="quiet"
      >
        {triggerIcon}
        {triggerText ? <span>{triggerText}</span> : null}
      </Button>
      <Popover
        className="c4-popover c4-menu-popover"
        offset={6}
        placement={align === "center" ? "bottom" : `bottom ${align}`}
      >
        <Menu aria-label={label} className="c4-menu">
          {items.map((item) => (
            <MenuItem
              id={item.id}
              {...(item.isDisabled ? { isDisabled: true } : {})}
              key={item.id}
              onAction={item.onAction}
              textValue={item.label}
            >
              <span className="c4-menu-item__copy">
                <span>{item.label}</span>
                {item.description ? <small>{item.description}</small> : null}
              </span>
              {item.shortcut ? (
                <kbd className="c4-menu-item__shortcut">{item.shortcut}</kbd>
              ) : null}
            </MenuItem>
          ))}
        </Menu>
      </Popover>
    </MenuTrigger>
  );
}

/** Renders a compact, non-modal detail popover with explicit dismissal. */
export function PopoverDialog({
  children,
  placement = "bottom",
  title,
  triggerIcon,
  triggerLabel,
  triggerText,
}: PopoverDialogProps) {
  return (
    <DialogTrigger>
      <Button
        {...(triggerText
          ? {}
          : { "aria-label": triggerLabel, className: "c4-icon-button" })}
        variant="quiet"
      >
        {triggerIcon}
        {triggerText ? <span>{triggerText}</span> : null}
      </Button>
      <Popover
        className="c4-popover c4-dialog-popover"
        offset={6}
        placement={placement}
      >
        <Dialog className="c4-popover-dialog">
          {({ close }) => (
            <>
              <header className="c4-overlay-header">
                <Heading slot="title">{title}</Heading>
                <Button
                  aria-label={`Close ${title}`}
                  className="c4-icon-button"
                  onPress={close}
                  variant="quiet"
                >
                  <Icon name="close" />
                </Button>
              </header>
              <div className="c4-overlay-body">{children}</div>
            </>
          )}
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}

/** Renders a focus-contained modal with Escape, backdrop, and focus restoration. */
export function ModalDialog({
  children,
  closeLabel = "Close",
  isDismissable = true,
  renderActions,
  title,
  triggerLabel,
  triggerVariant = "secondary",
}: ModalDialogProps) {
  return (
    <DialogTrigger>
      <Button variant={triggerVariant}>{triggerLabel}</Button>
      <ModalOverlay className="c4-modal-overlay" isDismissable={isDismissable}>
        <Modal className="c4-modal">
          <Dialog className="c4-modal-dialog">
            {({ close }) => (
              <>
                <header className="c4-overlay-header">
                  <Heading slot="title">{title}</Heading>
                  <Button
                    aria-label={`${closeLabel} ${title}`}
                    className="c4-icon-button"
                    isDisabled={!isDismissable}
                    onPress={close}
                    variant="quiet"
                  >
                    <Icon name="close" />
                  </Button>
                </header>
                <div className="c4-overlay-body">{children}</div>
                <footer className="c4-modal-dialog__actions">
                  {renderActions ? (
                    renderActions(close)
                  ) : (
                    <Button onPress={close}>{closeLabel}</Button>
                  )}
                </footer>
              </>
            )}
          </Dialog>
        </Modal>
      </ModalOverlay>
    </DialogTrigger>
  );
}

/** Renders an application-state-owned modal with the shared focus contract. */
export function ControlledModalDialog({
  children,
  closeLabel = "Close",
  isDismissable = true,
  isOpen,
  onDismiss,
  renderActions,
  title,
}: ControlledModalDialogProps) {
  return (
    <ModalOverlay
      className="c4-modal-overlay"
      isDismissable={isDismissable}
      isOpen={isOpen}
      onOpenChange={(open) => {
        if (!open) onDismiss();
      }}
    >
      <Modal className="c4-modal">
        <Dialog className="c4-modal-dialog">
          <header className="c4-overlay-header">
            <Heading slot="title">{title}</Heading>
            <Button
              aria-label={`${closeLabel} ${title}`}
              className="c4-icon-button"
              onPress={onDismiss}
              variant="quiet"
            >
              <Icon name="close" />
            </Button>
          </header>
          <div className="c4-overlay-body">{children}</div>
          <footer className="c4-modal-dialog__actions">
            {renderActions ? (
              renderActions()
            ) : (
              <Button onPress={onDismiss}>{closeLabel}</Button>
            )}
          </footer>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}
