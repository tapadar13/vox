import { useCallback, useEffect, useRef, useState } from "react";

import { onHistoryChanged, onState, vox } from "./tauri";
import { idleState, type AppState } from "./types";

export function useVoxState() {
  const [state, setState] = useState<AppState>(idleState);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let mounted = true;
    let unlisten = () => undefined;
    void vox
      .state()
      .then((next) => mounted && setState(next))
      .finally(() => mounted && setReady(true));
    void onState((next) => mounted && setState(next)).then((stop) => {
      if (mounted) unlisten = stop;
      else stop();
    });
    return () => {
      mounted = false;
      unlisten();
    };
  }, []);

  return { state, ready };
}

export function useHistoryVersion() {
  const [version, setVersion] = useState(0);

  useEffect(() => {
    let mounted = true;
    let unlisten = () => undefined;
    void onHistoryChanged(() => mounted && setVersion((value) => value + 1)).then((stop) => {
      if (mounted) unlisten = stop;
      else stop();
    });
    return () => {
      mounted = false;
      unlisten();
    };
  }, []);

  return version;
}

export function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timeout = window.setTimeout(() => setDebounced(value), delayMs);
    return () => window.clearTimeout(timeout);
  }, [delayMs, value]);
  return debounced;
}

export function useAsyncAction<TArgs extends unknown[]>(
  action: (...args: TArgs) => Promise<unknown>,
) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const run = useCallback(
    async (...args: TArgs) => {
      setBusy(true);
      setError(null);
      try {
        await action(...args);
      } catch (caught) {
        const message =
          typeof caught === "object" && caught && "message" in caught
            ? String(caught.message)
            : String(caught);
        if (mounted.current) setError(message);
      } finally {
        if (mounted.current) setBusy(false);
      }
    },
    [action],
  );

  return { run, busy, error, clearError: () => setError(null) };
}
