import type { FormEvent } from "react";
import { useId, useLayoutEffect, useRef, useState } from "react";
import {
  Button,
  Dialog,
  Heading,
  Menu,
  MenuItem,
  MenuSection,
  MenuTrigger,
  Modal,
  ModalOverlay,
  Popover,
} from "react-aria-components";

import { Icon } from "../../../components/accessible";
import type { GitBranchControlProps, GitBranchOption } from "./types";

const CREATE_BRANCH_KEY = "create-branch";
const INVALID_BRANCH_CHARACTERS = new Set(["~", "^", ":", "?", "*", "[", "\\"]);

/** Renders the projection-driven Git branch control and approval confirmation. */
export function GitBranchControl({
  activeBranch,
  branches,
  isBusy = false,
  onApprove,
  onCreate,
  onDeny,
  onSwitch,
  pendingApprovalId = null,
}: GitBranchControlProps) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shouldRestoreMenuFocus = useRef(false);
  const shouldRestoreDialogFocus = useRef(false);
  const branchErrorId = useId();
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [branchName, setBranchName] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);

  /** Tracks ordinary menu dismissal so focus returns to the Branch trigger. */
  const handleMenuOpenChange = (nextIsOpen: boolean) => {
    if (!nextIsOpen) {
      shouldRestoreMenuFocus.current = true;
    }
    setIsMenuOpen(nextIsOpen);
  };

  /** Reports one branch selection while leaving Git authority to the caller. */
  const handleBranchSelect = (branch: GitBranchOption) => {
    if (branch.name !== activeBranch) {
      onSwitch(branch.name);
    }
    shouldRestoreMenuFocus.current = true;
    setIsMenuOpen(false);
  };

  /** Moves from the menu into a controlled branch-name dialog. */
  const handleCreateOpen = () => {
    shouldRestoreMenuFocus.current = false;
    setBranchName("");
    setValidationError(null);
    setIsMenuOpen(false);
    setIsCreateDialogOpen(true);
  };

  /** Closes the create dialog and requests predictable trigger restoration. */
  const handleCreateOpenChange = (nextIsOpen: boolean) => {
    if (!nextIsOpen) {
      shouldRestoreDialogFocus.current = true;
      setValidationError(null);
    }
    setIsCreateDialogOpen(nextIsOpen);
  };

  /** Validates and reports a normalized branch name without performing Git work. */
  const handleCreateSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedName = branchName.trim();
    const error = branchNameError(normalizedName, branches);
    if (error) {
      setValidationError(error);
      return;
    }

    onCreate(normalizedName);
    handleCreateOpenChange(false);
  };

  useLayoutEffect(() => {
    if (!isMenuOpen && shouldRestoreMenuFocus.current) {
      shouldRestoreMenuFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [isMenuOpen]);

  useLayoutEffect(() => {
    if (!isCreateDialogOpen && shouldRestoreDialogFocus.current) {
      shouldRestoreDialogFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [isCreateDialogOpen]);

  return (
    <div className="conversation-branch-control">
      <MenuTrigger isOpen={isMenuOpen} onOpenChange={handleMenuOpenChange}>
        <Button
          aria-label="Git branch"
          className="conversation-branch-control__trigger"
          isDisabled={isBusy || pendingApprovalId !== null}
          ref={triggerRef}
        >
          <span
            aria-hidden="true"
            className="conversation-branch-control__mark"
          >
            ⎇
          </span>
          <span>{activeBranch}</span>
          <Icon aria-hidden="true" name="chevron-down" size={14} />
        </Button>
        <Popover
          className="conversation-branch-control__popover"
          offset={6}
          placement="top start"
        >
          <Menu
            aria-label="Local Git branches"
            className="conversation-branch-control__menu"
            shouldCloseOnSelect={false}
          >
            <MenuSection
              selectedKeys={[branchKey(activeBranch)]}
              selectionMode="single"
            >
              {branches.map((branch) => (
                <MenuItem
                  aria-label={`${branch.name}, ${shortOid(branch.targetOid)}`}
                  className="conversation-branch-control__branch"
                  id={branchKey(branch.name)}
                  key={branchKey(branch.name)}
                  onAction={() => handleBranchSelect(branch)}
                  textValue={branch.name}
                >
                  <span>{branch.name}</span>
                  <small>{shortOid(branch.targetOid)}</small>
                </MenuItem>
              ))}
            </MenuSection>
            <MenuSection>
              <MenuItem
                className="conversation-branch-control__create"
                id={CREATE_BRANCH_KEY}
                onAction={handleCreateOpen}
                textValue="Create New"
              >
                + Create New
              </MenuItem>
            </MenuSection>
          </Menu>
        </Popover>
      </MenuTrigger>

      {pendingApprovalId ? (
        <section
          aria-label="Branch change approval"
          className="conversation-branch-control__approval"
        >
          <span>Branch change requires approval.</span>
          <span className="conversation-branch-control__approval-actions">
            <Button
              className="c4-button c4-button--secondary"
              isDisabled={isBusy}
              onPress={() => onApprove(pendingApprovalId)}
            >
              Approve
            </Button>
            <Button
              className="c4-button c4-button--secondary"
              isDisabled={isBusy}
              onPress={() => onDeny(pendingApprovalId)}
            >
              Cancel
            </Button>
          </span>
        </section>
      ) : null}

      {isCreateDialogOpen ? (
        <ModalOverlay
          className="c4-modal-overlay"
          isDismissable={!isBusy}
          isOpen={isCreateDialogOpen}
          onOpenChange={handleCreateOpenChange}
        >
          <Modal className="c4-modal">
            <Dialog className="c4-modal-dialog">
              <form noValidate onSubmit={handleCreateSubmit}>
                <header className="c4-overlay-header">
                  <Heading slot="title">Create branch</Heading>
                  <Button
                    aria-label="Close Create branch"
                    className="c4-button c4-button--quiet c4-icon-button"
                    isDisabled={isBusy}
                    onPress={() => handleCreateOpenChange(false)}
                  >
                    <Icon aria-hidden="true" name="close" />
                  </Button>
                </header>
                <div className="c4-overlay-body">
                  <label
                    className="conversation-branch-control__field"
                    htmlFor={branchErrorId}
                  >
                    <span>Branch name</span>
                    <input
                      aria-describedby={
                        validationError ? `${branchErrorId}-error` : undefined
                      }
                      aria-invalid={validationError ? "true" : undefined}
                      autoFocus
                      disabled={isBusy}
                      id={branchErrorId}
                      maxLength={255}
                      onChange={(event) => {
                        setBranchName(event.currentTarget.value);
                        if (validationError) {
                          setValidationError(null);
                        }
                      }}
                      placeholder="feature/branch-name"
                      value={branchName}
                    />
                  </label>
                  {validationError ? (
                    <p
                      className="conversation-branch-control__error"
                      id={`${branchErrorId}-error`}
                      role="alert"
                    >
                      {validationError}
                    </p>
                  ) : null}
                </div>
                <footer className="c4-modal-dialog__actions">
                  <Button
                    className="c4-button c4-button--secondary"
                    isDisabled={isBusy}
                    onPress={() => handleCreateOpenChange(false)}
                    type="button"
                  >
                    Cancel
                  </Button>
                  <Button
                    className="c4-button c4-button--primary"
                    isDisabled={isBusy}
                    type="submit"
                  >
                    Create
                  </Button>
                </footer>
              </form>
            </Dialog>
          </Modal>
        </ModalOverlay>
      ) : null}
    </div>
  );
}

/** Creates a stable menu identity for a local branch name. */
function branchKey(branchName: string) {
  return JSON.stringify(["branch", branchName]);
}

/** Produces the compact target identifier shown beside each branch. */
function shortOid(targetOid: string) {
  return targetOid.slice(0, 8);
}

/** Validates the user-entered shape before asking the Git boundary to create it. */
function branchNameError(
  branchName: string,
  branches: readonly GitBranchOption[],
): string | null {
  if (!branchName) {
    return "Enter a branch name.";
  }
  if (branches.some((branch) => branch.name === branchName)) {
    return "A local branch with this name already exists.";
  }

  const hasInvalidShape =
    branchName === "@" ||
    branchName.startsWith("/") ||
    branchName.endsWith("/") ||
    branchName.endsWith(".") ||
    branchName.endsWith(".lock") ||
    branchName.includes("..") ||
    branchName.includes("//") ||
    branchName.includes("@{") ||
    branchName.split("/").some((part) => part.startsWith(".")) ||
    hasInvalidBranchCharacter(branchName);
  if (hasInvalidShape) {
    return "Use a valid Git branch name without spaces or Git ref punctuation.";
  }

  return null;
}

/** Finds whitespace, control characters, and Git-reserved ref punctuation. */
function hasInvalidBranchCharacter(branchName: string) {
  for (const character of branchName) {
    const codePoint = character.codePointAt(0) ?? 0;
    if (
      codePoint <= 0x20 ||
      codePoint === 0x7f ||
      INVALID_BRANCH_CHARACTERS.has(character)
    ) {
      return true;
    }
  }

  return false;
}
