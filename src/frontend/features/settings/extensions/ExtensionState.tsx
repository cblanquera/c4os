import { useId } from "react";

import { Button, Notice, StatusRegion } from "../../../components/accessible";

export interface ExtensionLoadingProps {
  readonly message: string;
  readonly title: string;
}

export interface ExtensionErrorProps {
  readonly message: string;
  readonly onRetry: () => void;
  readonly retryable: boolean;
  readonly title: string;
}

export interface ExtensionEmptyProps {
  readonly detail: string;
  readonly title: string;
}

/** Keeps asynchronous Settings projections explicit instead of rendering stale rows. */
export function ExtensionLoading({ message, title }: ExtensionLoadingProps) {
  return (
    <StatusRegion className="extension-state extension-state--loading">
      <span aria-hidden="true" className="extension-state__spinner" />
      <span>
        <strong>{title}</strong>
        <span>{message}</span>
      </span>
    </StatusRegion>
  );
}

/** Renders a service-owned failure with retry only when the projection allows it. */
export function ExtensionError({
  message,
  onRetry,
  retryable,
  title,
}: ExtensionErrorProps) {
  return (
    <Notice className="extension-state" title={title} tone="danger">
      <p>{message}</p>
      {retryable ? (
        <Button onPress={onRetry} variant="secondary">
          Try again
        </Button>
      ) : null}
    </Notice>
  );
}

/** Identifies a valid empty result separately from loading and failure. */
export function ExtensionEmpty({ detail, title }: ExtensionEmptyProps) {
  const titleId = useId();

  return (
    <section className="extension-empty" aria-labelledby={titleId}>
      <div aria-hidden="true" className="extension-empty__mark">
        +
      </div>
      <h2 id={titleId}>{title}</h2>
      <p>{detail}</p>
    </section>
  );
}
