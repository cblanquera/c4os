import { useLayoutEffect, useRef, useState } from "react";
import {
  Button,
  Dialog,
  DialogTrigger,
  Heading,
  Popover,
} from "react-aria-components";

import { Icon } from "../../../components/accessible";
import type { ChatInformation } from "./types";

export interface ChatInformationPopoverProps {
  readonly information: ChatInformation;
}

/** Renders the icon-only, focus-managed summary of the effective Chat route. */
export function ChatInformationPopover({
  information,
}: ChatInformationPopoverProps) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shouldRestoreFocus = useRef(false);
  const [isOpen, setIsOpen] = useState(false);
  const usedTokens = clamp(
    information.contextUsage.usedTokens,
    0,
    Math.max(0, information.contextUsage.totalTokens),
  );
  const totalTokens = Math.max(0, information.contextUsage.totalTokens);
  const remainingTokens = Math.max(0, totalTokens - usedTokens);
  const usedPercent = totalTokens === 0 ? 0 : (usedTokens * 100) / totalTokens;
  const displayedUsedPercent = Math.round(usedPercent);
  const displayedRemainingPercent = 100 - displayedUsedPercent;

  /** Tracks dismissal so focus returns to the icon-only trigger. */
  const handleOpenChange = (nextIsOpen: boolean) => {
    if (!nextIsOpen) {
      shouldRestoreFocus.current = true;
    }
    setIsOpen(nextIsOpen);
  };

  useLayoutEffect(() => {
    if (!isOpen && shouldRestoreFocus.current) {
      shouldRestoreFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [isOpen]);

  return (
    <span className="conversation-chat-information">
      <DialogTrigger isOpen={isOpen} onOpenChange={handleOpenChange}>
        <Button
          aria-label="Chat information"
          className="c4-button c4-button--quiet c4-icon-button"
          ref={triggerRef}
        >
          <Icon aria-hidden="true" name="info" size={16} />
        </Button>
        <Popover
          className="c4-popover c4-dialog-popover conversation-chat-information__popover"
          offset={6}
          placement="bottom"
        >
          <Dialog className="c4-popover-dialog">
            {({ close }) => (
              <>
                <header className="c4-overlay-header">
                  <Heading slot="title">Chat information</Heading>
                  <Button
                    aria-label="Close Chat information"
                    className="c4-button c4-button--quiet c4-icon-button"
                    onPress={close}
                  >
                    <Icon aria-hidden="true" name="close" />
                  </Button>
                </header>
                <div className="c4-overlay-body">
                  <div
                    className="conversation-chat-information__health"
                    role="status"
                  >
                    {information.health}
                  </div>
                  <dl className="conversation-chat-information__details">
                    <InformationRow
                      label="Runtime"
                      value={information.runtime}
                    />
                    <InformationRow
                      label="Environment"
                      value={information.environment}
                    />
                    <InformationRow
                      label="Workspace"
                      value={information.workspace}
                    />
                    <InformationRow label="Model" value={information.model} />
                    <div className="conversation-chat-information__context">
                      <dt>Context window</dt>
                      <dd>
                        <span className="conversation-chat-information__context-summary">
                          <strong>
                            {formatPercent(displayedUsedPercent)} used
                          </strong>
                          <span aria-hidden="true"> · </span>
                          <span>
                            {formatPercent(displayedRemainingPercent)} left
                          </span>
                        </span>
                        <small>
                          {formatTokens(usedTokens)} /{" "}
                          {formatTokens(totalTokens)}
                          {" tokens"}
                        </small>
                        <span
                          aria-label="Context window used"
                          aria-valuemax={totalTokens}
                          aria-valuemin={0}
                          aria-valuenow={usedTokens}
                          aria-valuetext={`${formatTokens(usedTokens)} of ${formatTokens(totalTokens)} tokens used; ${formatTokens(remainingTokens)} tokens remaining`}
                          className="conversation-chat-information__progress"
                          role="progressbar"
                        >
                          <span style={{ inlineSize: `${usedPercent}%` }} />
                        </span>
                      </dd>
                    </div>
                  </dl>
                </div>
              </>
            )}
          </Dialog>
        </Popover>
      </DialogTrigger>
    </span>
  );
}

interface InformationRowProps {
  readonly label: string;
  readonly value: string;
}

/** Renders one definition row in the Chat information summary. */
function InformationRow({ label, value }: InformationRowProps) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

/** Keeps context counters inside their authoritative non-negative range. */
function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}

/** Formats compact token counts without changing their progressbar values. */
function formatTokens(tokens: number) {
  return new Intl.NumberFormat("en", {
    maximumFractionDigits: tokens >= 100_000 ? 0 : 1,
    notation: "compact",
  }).format(tokens);
}

/** Formats bounded whole percentages for the compact summary. */
function formatPercent(value: number) {
  return `${Math.round(value)}%`;
}
