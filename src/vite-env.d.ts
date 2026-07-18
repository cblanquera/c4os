/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_C4OS_QA_FIXTURES?: "1";
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
