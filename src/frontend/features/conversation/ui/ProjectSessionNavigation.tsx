import {
  useEffect,
  useRef,
  useState,
  type DragEvent,
  type KeyboardEvent,
} from "react";

import { Icon } from "../../../components/accessible";
import type { ProjectNavigationItem } from "./types";
import "./conversation-ui.css";

export interface ProjectSessionNavigationProps {
  readonly activeProjectId: string | null;
  readonly activeSessionId: string | null;
  readonly onAddProject: () => void;
  readonly onNewChat: (projectId: string) => void;
  readonly onProjectActivate: (projectId: string) => void;
  readonly onProjectExpansionChange: (
    projectId: string,
    isExpanded: boolean,
  ) => void;
  readonly onProjectCopyPath: (projectId: string) => void;
  readonly onProjectOrderChange: (projectIds: readonly string[]) => void;
  readonly onProjectRelocate: (projectId: string) => void;
  readonly onProjectRemove: (projectId: string) => void;
  readonly onProjectRename: (projectId: string, displayName: string) => void;
  readonly onProjectReveal: (projectId: string) => void;
  readonly onSearchClear: () => void;
  readonly onSearchQueryChange: (query: string) => void;
  readonly onSessionActivate: (sessionId: string, projectId: string) => void;
  readonly onSessionRemove: (sessionId: string, projectId: string) => void;
  readonly projects: readonly ProjectNavigationItem[];
  readonly searchQuery: string;
}

/** Renders ordered projects or the accepted flat session-search replacement. */
export function ProjectSessionNavigation({
  activeProjectId,
  activeSessionId,
  onAddProject,
  onNewChat,
  onProjectActivate,
  onProjectCopyPath,
  onProjectExpansionChange,
  onProjectOrderChange,
  onProjectRelocate,
  onProjectRemove,
  onProjectRename,
  onProjectReveal,
  onSearchClear,
  onSearchQueryChange,
  onSessionActivate,
  onSessionRemove,
  projects,
  searchQuery,
}: ProjectSessionNavigationProps) {
  const searchInput = useRef<HTMLInputElement>(null);
  const openProjectMenu = useRef<HTMLDivElement>(null);
  const renameInput = useRef<HTMLInputElement>(null);
  const projectMenuTriggers = useRef(new Map<string, HTMLButtonElement>());
  const [openProjectMenuId, setOpenProjectMenuId] = useState<string | null>(
    null,
  );
  const [draggedProjectId, setDraggedProjectId] = useState<string | null>(null);
  const [renameProject, setRenameProject] = useState<{
    readonly id: string;
    readonly name: string;
  } | null>(null);
  const normalizedQuery = searchQuery.trim().toLocaleLowerCase();
  const isSearching = normalizedQuery.length > 0;
  const searchResults = isSearching
    ? projects.flatMap((project) =>
        project.sessions
          .filter((session) =>
            session.title.toLocaleLowerCase().includes(normalizedQuery),
          )
          .map((session) => ({ project, session })),
      )
    : [];

  useEffect(() => {
    if (openProjectMenuId === null) return;
    openProjectMenu.current
      ?.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')
      ?.focus();
  }, [openProjectMenuId]);

  useEffect(() => {
    if (renameProject === null) return;
    renameInput.current?.focus();
    renameInput.current?.select();
  }, [renameProject]);

  /** Clears only the query and returns focus without disturbing navigation state. */
  function clearSearch() {
    onSearchQueryChange("");
    onSearchClear();
    searchInput.current?.focus();
  }

  /** Gives Escape the same explicit restoration behavior as the clear action. */
  function handleSearchKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key !== "Escape" || !isSearching) return;
    event.preventDefault();
    event.stopPropagation();
    clearSearch();
  }

  /** Closes a Project action menu and restores its trigger when it survives. */
  function closeProjectActions(shouldRestoreFocus = true) {
    const projectId = openProjectMenuId;
    setOpenProjectMenuId(null);
    if (shouldRestoreFocus && projectId !== null) {
      projectMenuTriggers.current.get(projectId)?.focus();
    }
  }

  /** Runs one explicit Project action through the owning service callback. */
  function runProjectAction(action: () => void, restoresFocus = true) {
    closeProjectActions(restoresFocus);
    action();
  }

  /** Closes the controlled rename modal and restores its surviving trigger. */
  function closeRenameDialog() {
    const projectId = renameProject?.id ?? null;
    setRenameProject(null);
    if (projectId !== null) {
      projectMenuTriggers.current.get(projectId)?.focus();
    }
  }

  /** Keeps keyboard focus inside the modal until an explicit close action. */
  function handleRenameDialogKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeRenameDialog();
      return;
    }
    if (event.key !== "Tab") return;
    const focusable = Array.from(
      event.currentTarget.querySelectorAll<HTMLElement>(
        'input, button:not(:disabled), [tabindex]:not([tabindex="-1"])',
      ),
    );
    const first = focusable.at(0);
    const last = focusable.at(-1);
    if (first === undefined || last === undefined) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  /** Moves the entire Project record while leaving its nested sessions intact. */
  function reorderProject(projectId: string, direction: -1 | 1) {
    const currentIndex = projects.findIndex(
      (project) => project.id === projectId,
    );
    const nextIndex = currentIndex + direction;
    if (currentIndex < 0 || nextIndex < 0 || nextIndex >= projects.length) {
      return;
    }

    const reorderedProjects = [...projects];
    const [project] = reorderedProjects.splice(currentIndex, 1);
    if (project === undefined) return;
    reorderedProjects.splice(nextIndex, 0, project);
    onProjectOrderChange(reorderedProjects.map((item) => item.id));
  }

  /** Reorders the full Project record at one explicit native drag target. */
  function dropProject(
    event: DragEvent<HTMLLIElement>,
    targetProjectId: string,
  ) {
    event.preventDefault();
    const sourceProjectId = draggedProjectId;
    setDraggedProjectId(null);
    if (sourceProjectId === null || sourceProjectId === targetProjectId) return;
    const sourceIndex = projects.findIndex(
      (project) => project.id === sourceProjectId,
    );
    const targetIndex = projects.findIndex(
      (project) => project.id === targetProjectId,
    );
    if (sourceIndex < 0 || targetIndex < 0) return;
    const reordered = [...projects];
    const [source] = reordered.splice(sourceIndex, 1);
    if (source === undefined) return;
    reordered.splice(targetIndex, 0, source);
    onProjectOrderChange(reordered.map((project) => project.id));
  }

  /** Supports direct keyboard reordering from the stable Project-name control. */
  function handleProjectKeyDown(
    event: KeyboardEvent<HTMLButtonElement>,
    projectId: string,
  ) {
    if (!event.altKey || !["ArrowUp", "ArrowDown"].includes(event.key)) return;
    event.preventDefault();
    reorderProject(projectId, event.key === "ArrowUp" ? -1 : 1);
  }

  /** Keeps the open action menu operable with standard menu navigation keys. */
  function handleProjectMenuKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeProjectActions();
      return;
    }

    const items = Array.from(
      event.currentTarget.querySelectorAll<HTMLButtonElement>(
        '[role="menuitem"]:not(:disabled)',
      ),
    );
    if (items.length === 0) return;

    const activeIndex = items.indexOf(
      document.activeElement as HTMLButtonElement,
    );
    let nextIndex: number | null = null;
    if (event.key === "ArrowDown") {
      nextIndex = activeIndex < 0 ? 0 : (activeIndex + 1) % items.length;
    } else if (event.key === "ArrowUp") {
      nextIndex =
        activeIndex < 0
          ? items.length - 1
          : (activeIndex - 1 + items.length) % items.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = items.length - 1;
    }

    if (nextIndex === null) return;
    event.preventDefault();
    items[nextIndex]?.focus();
  }

  return (
    <nav
      aria-label="Projects and chat sessions"
      className="conversation-navigation"
    >
      <div className="conversation-navigation__search" role="search">
        <Icon aria-hidden="true" name="search" size={14} />
        <label
          className="conversation-visually-hidden"
          htmlFor="conversation-session-search"
        >
          Search chat sessions
        </label>
        <div className="conversation-navigation__search-control">
          <input
            id="conversation-session-search"
            autoComplete="off"
            placeholder="Search chat sessions"
            ref={searchInput}
            type="search"
            value={searchQuery}
            onChange={(event) => onSearchQueryChange(event.currentTarget.value)}
            onKeyDown={handleSearchKeyDown}
          />
          {isSearching ? (
            <button
              type="button"
              aria-label="Clear chat search"
              onClick={clearSearch}
            >
              <Icon aria-hidden="true" name="close" size={14} />
            </button>
          ) : null}
        </div>
      </div>

      {isSearching ? (
        <section
          className="conversation-navigation__replacement"
          aria-labelledby="conversation-search-results-title"
        >
          <h2 id="conversation-search-results-title">Search results</h2>
          {searchResults.length === 0 ? (
            <p className="conversation-navigation__empty" role="status">
              No chat sessions found.
            </p>
          ) : (
            <ul className="conversation-navigation__results">
              {searchResults.map(({ project, session }) => (
                <li
                  key={session.id}
                  className="conversation-navigation__result-row"
                >
                  <button
                    type="button"
                    aria-current={
                      session.id === activeSessionId ? "page" : undefined
                    }
                    onClick={() => onSessionActivate(session.id, project.id)}
                  >
                    <strong>{session.title}</strong>
                    <span>{project.name}</span>
                  </button>
                  <button
                    type="button"
                    className="conversation-navigation__session-remove"
                    aria-label={`Remove ${session.title} from ${project.name}`}
                    onClick={() => onSessionRemove(session.id, project.id)}
                  >
                    Remove
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      ) : (
        <section
          className="conversation-navigation__projects"
          aria-labelledby="conversation-projects-title"
        >
          <header>
            <h2 id="conversation-projects-title">Projects</h2>
            <button
              type="button"
              aria-label="Add project"
              onClick={onAddProject}
            >
              <Icon aria-hidden="true" name="add" size={14} />
            </button>
          </header>
          {projects.length === 0 ? (
            <p className="conversation-navigation__empty">
              No projects are open.
            </p>
          ) : (
            <ol className="conversation-navigation__project-list">
              {projects.map((project) => {
                const sessionsId = `project-${project.id}-sessions`;
                const menuId = `project-${project.id}-actions`;
                const isActive = project.id === activeProjectId;
                const projectIndex = projects.findIndex(
                  (candidate) => candidate.id === project.id,
                );
                const isMenuOpen = openProjectMenuId === project.id;
                return (
                  <li
                    key={project.id}
                    draggable
                    data-active={isActive}
                    data-dragging={draggedProjectId === project.id}
                    data-path-state={project.pathState}
                    onDragEnd={() => setDraggedProjectId(null)}
                    onDragOver={(event) => {
                      if (draggedProjectId === null) return;
                      event.preventDefault();
                      event.dataTransfer.dropEffect = "move";
                    }}
                    onDragStart={(event) => {
                      event.dataTransfer.effectAllowed = "move";
                      event.dataTransfer.setData("text/plain", project.id);
                      setDraggedProjectId(project.id);
                    }}
                    onDrop={(event) => dropProject(event, project.id)}
                  >
                    <div className="conversation-navigation__project-row">
                      <button
                        type="button"
                        className="conversation-navigation__disclosure"
                        aria-controls={sessionsId}
                        aria-expanded={project.isExpanded}
                        aria-label={`${project.isExpanded ? "Collapse" : "Expand"} ${project.name}`}
                        onClick={() =>
                          onProjectExpansionChange(
                            project.id,
                            !project.isExpanded,
                          )
                        }
                      >
                        <span aria-hidden="true">
                          {project.isExpanded ? "⌄" : "›"}
                        </span>
                      </button>
                      <button
                        type="button"
                        className="conversation-navigation__project-name"
                        aria-current={isActive ? "true" : undefined}
                        aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown"
                        title="Activate project. Press Alt+Up or Alt+Down to reorder."
                        onClick={() => onProjectActivate(project.id)}
                        onKeyDown={(event) =>
                          handleProjectKeyDown(event, project.id)
                        }
                      >
                        <Icon aria-hidden="true" name="project" size={14} />
                        <span>{project.name}</span>
                        {project.pathState === "missing" ? (
                          <span className="conversation-visually-hidden">
                            Project folder is missing
                          </span>
                        ) : null}
                      </button>
                      <div
                        className="conversation-navigation__project-actions"
                        data-open={isMenuOpen}
                        aria-label={`${project.name} actions`}
                      >
                        <button
                          type="button"
                          aria-label={`New chat in ${project.name}`}
                          onClick={() => onNewChat(project.id)}
                        >
                          <Icon aria-hidden="true" name="add" size={14} />
                        </button>
                        <button
                          type="button"
                          aria-label={`More actions for ${project.name}`}
                          aria-controls={menuId}
                          aria-expanded={isMenuOpen}
                          aria-haspopup="menu"
                          ref={(element) => {
                            if (element === null) {
                              projectMenuTriggers.current.delete(project.id);
                            } else {
                              projectMenuTriggers.current.set(
                                project.id,
                                element,
                              );
                            }
                          }}
                          onClick={() =>
                            setOpenProjectMenuId(isMenuOpen ? null : project.id)
                          }
                        >
                          <Icon aria-hidden="true" name="more" size={14} />
                        </button>
                        {isMenuOpen ? (
                          <div
                            id={menuId}
                            ref={openProjectMenu}
                            className="conversation-navigation__project-menu"
                            role="menu"
                            aria-label={`Actions for ${project.name}`}
                            onKeyDown={handleProjectMenuKeyDown}
                          >
                            {project.pathState === "found" ? (
                              <button
                                type="button"
                                role="menuitem"
                                onClick={() =>
                                  runProjectAction(() =>
                                    onProjectReveal(project.id),
                                  )
                                }
                              >
                                Reveal
                              </button>
                            ) : (
                              <button
                                type="button"
                                role="menuitem"
                                onClick={() =>
                                  runProjectAction(() =>
                                    onProjectRelocate(project.id),
                                  )
                                }
                              >
                                Relocate
                              </button>
                            )}
                            <button
                              type="button"
                              role="menuitem"
                              onClick={() =>
                                runProjectAction(() =>
                                  onProjectCopyPath(project.id),
                                )
                              }
                            >
                              Copy path
                            </button>
                            <button
                              type="button"
                              role="menuitem"
                              onClick={() =>
                                runProjectAction(() =>
                                  setRenameProject({
                                    id: project.id,
                                    name: project.name,
                                  }),
                                )
                              }
                            >
                              Rename
                            </button>
                            <button
                              type="button"
                              role="menuitem"
                              onClick={() =>
                                runProjectAction(
                                  () => onProjectRemove(project.id),
                                  false,
                                )
                              }
                            >
                              Remove
                            </button>
                            <hr role="separator" />
                            <button
                              type="button"
                              role="menuitem"
                              disabled={projectIndex === 0}
                              onClick={() =>
                                runProjectAction(() =>
                                  reorderProject(project.id, -1),
                                )
                              }
                            >
                              Move up
                            </button>
                            <button
                              type="button"
                              role="menuitem"
                              disabled={projectIndex === projects.length - 1}
                              onClick={() =>
                                runProjectAction(() =>
                                  reorderProject(project.id, 1),
                                )
                              }
                            >
                              Move down
                            </button>
                          </div>
                        ) : null}
                      </div>
                    </div>
                    {project.isExpanded ? (
                      <ul
                        id={sessionsId}
                        className="conversation-navigation__sessions"
                      >
                        {project.sessions.map((session) => (
                          <li
                            key={session.id}
                            className="conversation-navigation__session-row"
                          >
                            <button
                              type="button"
                              className="conversation-navigation__session-name"
                              aria-current={
                                session.id === activeSessionId
                                  ? "page"
                                  : undefined
                              }
                              onClick={() =>
                                onSessionActivate(session.id, project.id)
                              }
                            >
                              {session.title}
                            </button>
                            <button
                              type="button"
                              className="conversation-navigation__session-remove"
                              aria-label={`Remove ${session.title} from ${project.name}`}
                              onClick={() =>
                                onSessionRemove(session.id, project.id)
                              }
                            >
                              <Icon aria-hidden="true" name="close" size={14} />
                            </button>
                          </li>
                        ))}
                      </ul>
                    ) : null}
                  </li>
                );
              })}
            </ol>
          )}
        </section>
      )}
      {renameProject ? (
        <div
          aria-labelledby="conversation-project-rename-title"
          aria-modal="true"
          className="conversation-navigation__rename-backdrop"
          role="dialog"
          onClick={(event) => {
            if (event.target === event.currentTarget) closeRenameDialog();
          }}
          onKeyDown={handleRenameDialogKeyDown}
        >
          <form
            className="conversation-navigation__rename-dialog"
            onSubmit={(event) => {
              event.preventDefault();
              const displayName = renameInput.current?.value.trim() ?? "";
              if (displayName.length === 0) return;
              onProjectRename(renameProject.id, displayName);
              closeRenameDialog();
            }}
          >
            <h2 id="conversation-project-rename-title">Rename Project</h2>
            <label>
              Project name
              <input
                ref={renameInput}
                autoComplete="off"
                defaultValue={renameProject.name}
                required
              />
            </label>
            <div>
              <button type="button" onClick={closeRenameDialog}>
                Cancel
              </button>
              <button type="submit">Rename</button>
            </div>
          </form>
        </div>
      ) : null}
    </nav>
  );
}
