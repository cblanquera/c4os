import { invoke } from "@tauri-apps/api/core";

import { createTauriAdapter } from "./tauri-adapter";

const foundationAdapter = createTauriAdapter({
  invoke(command, args) {
    return invoke(command, args);
  },
});

/** Read the Rust-owned foundation snapshot through the sole allowlisted command. */
export function readFoundationSnapshot() {
  return foundationAdapter.readFoundationSnapshot();
}
