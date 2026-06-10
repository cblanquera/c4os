// Local-only diagnostics baseline for the MVP.
export const DIAGNOSTICS_STORAGE_MODE = 'local-only';

/**
 * Creates the diagnostics state exposed by the initial backend status command.
 */
export function createLocalDiagnostics() {
  return {
    storageMode: DIAGNOSTICS_STORAGE_MODE,
    productTelemetry: 'disabled',
    automaticCrashUpload: false,
    supportBundleUpload: false,
  };
}
