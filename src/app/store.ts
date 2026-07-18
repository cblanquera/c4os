import { configureStore, createSlice } from "@reduxjs/toolkit";

type FoundationState = {
  readonly phase: "foundation";
};

const foundationSlice = createSlice({
  name: "foundation",
  initialState: { phase: "foundation" } satisfies FoundationState,
  reducers: {},
});

export const store = configureStore({
  reducer: {
    foundation: foundationSlice.reducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
