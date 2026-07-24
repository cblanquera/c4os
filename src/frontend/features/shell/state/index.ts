export {
  initialShellAuthorityState,
  shellAuthorityActions,
  shellAuthorityReducer,
} from "./authority";
export {
  initialShellDraftState,
  shellPanelBounds,
  shellDraftActions,
  shellDraftReducer,
} from "./drafts";
export {
  createBuildGatedQaPreloadedState,
  initialQaState,
  qaActions,
  qaReducer,
} from "./qa-fixtures";
export * from "./selectors";
export type * from "./types";
export { UNINITIALIZED_GENERATION } from "./types";

import { shellAuthorityReducer } from "./authority";
import { shellDraftReducer } from "./drafts";
import { qaReducer } from "./qa-fixtures";

/** Reducer map for composition into the application store. */
export const shellStateReducers = {
  shellAuthority: shellAuthorityReducer,
  shellDrafts: shellDraftReducer,
  shellQa: qaReducer,
};
