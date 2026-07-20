import { combineReducers, configureStore, createSlice } from "@reduxjs/toolkit";

import {
  createBuildGatedQaPreloadedState,
  shellStateReducers,
  type ShellPreloadedState,
} from "../features/shell/state";

type FoundationState = {
  readonly phase: "foundation";
};

const foundationSlice = createSlice({
  name: "foundation",
  initialState: { phase: "foundation" } satisfies FoundationState,
  reducers: {},
});

const rootReducer = combineReducers({
  foundation: foundationSlice.reducer,
  ...shellStateReducers,
});
export function createAppStore(
  shellPreloadedState:
    ShellPreloadedState | undefined = createBuildGatedQaPreloadedState(),
) {
  const preloadedState =
    shellPreloadedState === undefined
      ? undefined
      : {
          foundation: { phase: "foundation" as const },
          ...shellPreloadedState,
        };

  return configureStore({
    reducer: rootReducer,
    ...(preloadedState === undefined ? {} : { preloadedState }),
  });
}

export const store = createAppStore();

export type RootState = ReturnType<typeof rootReducer>;
export type AppDispatch = typeof store.dispatch;
export type AppStore = ReturnType<typeof createAppStore>;
