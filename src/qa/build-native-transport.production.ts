/** Unreachable production stub kept free of fixture code and data. */
export function invokeQaProductRoute(
  _command: string,
  _args: Readonly<Record<string, unknown>>,
): Promise<unknown> {
  void _command;
  void _args;
  return Promise.reject(new Error("Alternate renderer transport unavailable"));
}

/** Production shells always subscribe to native menu/window events. */
export function shouldSubscribeToNativeShellEvents(): boolean {
  return true;
}
