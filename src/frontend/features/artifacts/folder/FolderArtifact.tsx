import { useEffect, useRef } from "react";

import { ArtifactShell } from "../ui/ArtifactShell";
import type {
  ArtifactBreadcrumb,
  ArtifactContext,
  ArtifactShellStatus,
} from "../types";
import "./folder-artifact.css";

export interface FolderArtifactEntry {
  readonly id: string;
  readonly kind: "file" | "folder";
  readonly metadata?: string;
  readonly name: string;
}

export type FolderListingState =
  | { readonly phase: "ready" }
  | { readonly message: string; readonly phase: "loading" }
  | { readonly message: string; readonly phase: "error" };

export interface FolderArtifactModel {
  readonly artifactId: string;
  readonly breadcrumbs: readonly ArtifactBreadcrumb[];
  readonly entries: readonly FolderArtifactEntry[];
  readonly listing: FolderListingState;
  readonly listingLimit: number;
  readonly selectedEntryId?: string | undefined;
  readonly status?: ArtifactShellStatus;
  readonly title: string;
}

export interface FolderArtifactProps {
  readonly context: ArtifactContext;
  readonly model: FolderArtifactModel;
  readonly onBreadcrumbSelect?: (
    artifactId: string,
    breadcrumbId: string,
  ) => void;
  readonly onClose?: (artifactId: string) => void;
  readonly onConvertFile?: (artifactId: string, entryId: string) => void;
  readonly onCopy?: (artifactId: string, visibleListing: string) => void;
  readonly onExpand?: (artifactId: string) => void;
  readonly onNavigateFolder?: (artifactId: string, entryId: string) => void;
  readonly onRefresh?: (artifactId: string) => void;
  readonly onReply?: (
    artifactId: string,
    selection?: { readonly selectedEntryId: string },
  ) => void;
}

/** Renders a bounded, controlled Folder listing through the shared shell. */
export function FolderArtifact({
  context,
  model,
  onBreadcrumbSelect,
  onClose,
  onConvertFile,
  onCopy,
  onExpand,
  onNavigateFolder,
  onRefresh,
  onReply,
}: FolderArtifactProps) {
  const lastFocusedEntryId = useRef<string | undefined>(model.selectedEntryId);
  const safeLimit = Math.max(0, Math.floor(model.listingLimit));
  const visibleEntries = model.entries.slice(0, safeLimit);
  useEffect(() => {
    const retainedEntryId = lastFocusedEntryId.current;
    if (
      retainedEntryId !== undefined &&
      !model.entries.some((entry) => entry.id === retainedEntryId)
    ) {
      lastFocusedEntryId.current = model.entries.some(
        (entry) => entry.id === model.selectedEntryId,
      )
        ? model.selectedEntryId
        : undefined;
    }
  }, [model.entries, model.selectedEntryId]);
  const omittedCount = Math.max(
    0,
    model.entries.length - visibleEntries.length,
  );
  const listingCopy = visibleEntries
    .map((entry) => `${entry.kind}: ${entry.name}`)
    .join("\n");
  const status =
    model.status ??
    (model.listing.phase === "loading"
      ? ({ kind: "loading", message: model.listing.message } as const)
      : model.listing.phase === "error"
        ? ({ kind: "error", message: model.listing.message } as const)
        : ({ kind: "ready" } as const));
  const isContextual = context === "contextual";

  return (
    <ArtifactShell
      context={context}
      focusSupported
      footerContent={
        omittedCount > 0
          ? `${visibleEntries.length} shown · ${omittedCount} omitted by listing limit`
          : `${visibleEntries.length} ${visibleEntries.length === 1 ? "item" : "items"}`
      }
      headerContent={
        <div className="artifact-folder__header">
          <FolderBreadcrumbs
            artifactId={model.artifactId}
            breadcrumbs={model.breadcrumbs}
            isReadOnly={isContextual}
            onSelect={onBreadcrumbSelect}
          />
          <button
            disabled={onRefresh === undefined || isContextual}
            onClick={() => onRefresh?.(model.artifactId)}
            type="button"
          >
            Refresh
          </button>
        </div>
      }
      identity={{
        accessibleLabel: `Folder response artifact: ${model.title}`,
        id: model.artifactId,
        title: model.title,
        typeLabel: "Folder",
      }}
      onClose={onClose ? () => onClose(model.artifactId) : undefined}
      onCopy={onCopy ? () => onCopy(model.artifactId, listingCopy) : undefined}
      onExpand={onExpand ? () => onExpand(model.artifactId) : undefined}
      onReply={
        onReply
          ? () => {
              const selectedEntryId = [
                lastFocusedEntryId.current,
                model.selectedEntryId,
              ].find(
                (entryId) =>
                  entryId !== undefined &&
                  model.entries.some((entry) => entry.id === entryId),
              );
              if (selectedEntryId === undefined) {
                onReply(model.artifactId);
              } else {
                onReply(model.artifactId, { selectedEntryId });
              }
            }
          : undefined
      }
      status={status}
    >
      <FolderListing
        artifactId={model.artifactId}
        entries={visibleEntries}
        onConvertFile={onConvertFile}
        onEntryFocus={(entryId) => {
          lastFocusedEntryId.current = entryId;
        }}
        onNavigateFolder={onNavigateFolder}
        selectedEntryId={model.selectedEntryId}
      />
    </ArtifactShell>
  );
}

interface FolderBreadcrumbsProps {
  readonly artifactId: string;
  readonly breadcrumbs: readonly ArtifactBreadcrumb[];
  readonly isReadOnly: boolean;
  readonly onSelect: FolderArtifactProps["onBreadcrumbSelect"];
}

/** Renders folder path navigation while contextual Chat stays read-oriented. */
function FolderBreadcrumbs({
  artifactId,
  breadcrumbs,
  isReadOnly,
  onSelect,
}: FolderBreadcrumbsProps) {
  return (
    <nav
      aria-label="Folder breadcrumbs"
      className="artifact-folder__breadcrumbs"
    >
      <ol>
        {breadcrumbs.map((breadcrumb) => (
          <li key={breadcrumb.id}>
            <button
              aria-current={breadcrumb.isCurrent ? "page" : undefined}
              disabled={
                isReadOnly || breadcrumb.isCurrent || onSelect === undefined
              }
              onClick={() => onSelect?.(artifactId, breadcrumb.id)}
              type="button"
            >
              {breadcrumb.label}
            </button>
          </li>
        ))}
      </ol>
    </nav>
  );
}

interface FolderListingProps {
  readonly artifactId: string;
  readonly entries: readonly FolderArtifactEntry[];
  readonly onConvertFile: FolderArtifactProps["onConvertFile"];
  readonly onEntryFocus: (entryId: string) => void;
  readonly onNavigateFolder: FolderArtifactProps["onNavigateFolder"];
  readonly selectedEntryId: string | undefined;
}

/** Emits navigation or File conversion intent from a bounded semantic list. */
function FolderListing({
  artifactId,
  entries,
  onConvertFile,
  onEntryFocus,
  onNavigateFolder,
  selectedEntryId,
}: FolderListingProps) {
  if (entries.length === 0) {
    return <p className="artifact-folder__empty">This folder is empty.</p>;
  }

  return (
    <ul aria-label="Folder contents" className="artifact-folder__entries">
      {entries.map((entry) => {
        const action =
          entry.kind === "folder" ? onNavigateFolder : onConvertFile;
        const actionLabel =
          entry.kind === "folder"
            ? `Open folder ${entry.name}`
            : `Open file ${entry.name}`;
        return (
          <li
            data-kind={entry.kind}
            data-selected={entry.id === selectedEntryId}
            key={entry.id}
          >
            <button
              aria-label={actionLabel}
              disabled={action === undefined}
              onFocus={() => onEntryFocus(entry.id)}
              onClick={() => action?.(artifactId, entry.id)}
              type="button"
            >
              <span aria-hidden="true">
                {entry.kind === "folder" ? "▸" : "·"}
              </span>
              <strong>{entry.name}</strong>
              {entry.metadata ? <small>{entry.metadata}</small> : null}
            </button>
          </li>
        );
      })}
    </ul>
  );
}
