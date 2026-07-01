Code Refactor:
- I want to transform the code to be more modular where plugins activate the Tauri backend
- The persistent app shell should still be chat prompt, chat thread and all the settings (left panel basic chat history with no assigned project, no right panel by default)
- Plugin: File system (FS) - Activates Workspaces, Projects and chat items per project. Can be right or left panel based on config.
- Plugin: File editor (IDE) - Requires FS plugin. Activates file explorer and editor. Can be right or left panel based on config.
- Plugin: Terminal (CLI) - Activates user terminal per chat session. Can be right or left panel based on config.
- Plugin: Chat Debug - This is the former agent terminal. Can be right or left panel based on config.
- Plugin: Browser - Activates In Browser/Preview per chat session. Can be right or left panel based on config.
- Frontend - No right panel by default. One singular header (with plugin icons to the far left or right based on each config. Ability to reorder icons.). Clicking plugin icons will toggle show the plugin panel view. Clicking an active plugin icon will close the relative panel view. Both active left and right panel should be able resize up to the center pane fixed minimum width (set a fix minimum width for center pane)
- Plugins should be visible in Settings: plugins. Should follow Codex plugin system. Should have a custom `[plugin root]/agents/c4os.yaml` that describes its application on the c4os app shell. Active plugin configuration should live in ~/.c4os (provision location logic for windows as well). Any other c4os related code should live in `[plugin root]/c4os`

Plugin: File system (FS)
- Menu OS: Add "Create New Workspace"
- Workspace - Do not save project config in .c4os folder instead ~/.c4os with all the project and workspace configs (provision location logic for windows as well)
- Replace "Projects" label with name of workspace. Defaults to "Projects"
- Add project (+) should now show popover. Choose folder, Clone Repository
- Clone repository - form inputs: Repository URL, Local Folder Destination
- Ability to remove chat item - Dialog confirm, then remove (do not retain history)
- Ability to remove project - Dialog confirm, then remove (do not retain history)
- Ability to re-order projects (drag/drop)
- Clicking project label collapse/expands chat items
- Project options - (... icon) - on click, opens popover. Move remove project here. Order: Copy Path, Reveal In Finder (provision windows), Rename, Remove. 
- Search projects - basic search through chat threads. Results should take over center screen with an X on the top right
- Activating browser should not create a chat session item ("Browser session")Plugin: File editor (IDE) 
- File Explorer - add right-click context menu. Copy Path, Add to chat (dependent on prompt tagging task)
- File Explorer - Better file icons for common file typesPlugin: Chat Debug- Also show app tool calls (and parameters used)

Plugin: Browser 
- Browser Navigation - Back, Forward, Refresh
- Screenshot icon - Screenshot should automatically attach to chat prompt
- Annotations - Right click anywhere to add an annotation. On send, automatically attach to chat prompt
- .docx, xlsx preview

No more: 
- Start screen
- Right panel tabs (and right panel collapse icon)

App Shell:
- Chat Prompt - Approval Popover & Dialog (make it work)
- Chat Prompt - Choose/Create branch popover. If no .git, then don’t show branch button. Chat thread branch remains read-only
- Chat Prompt - Make attachments work (dependent on model research)
- Chat Prompt: Prompt Tagging - Add `$` for skills and `@` for plugin/files, `/` for agent cli

Settings:
- Skills - Pre-populate skill creator skill
- Runtime - Implement Pi runtime (first do proof with existing app layer in order to itemize out the adjustments needed)
- Configuration - Make config.toml useful (see: Codex and generally follow)
- Configuration - Remove Sandbox Settings
- Configuration - Instead of a single default Approval policy, make a default policy item per app tool and explain what each app tool does