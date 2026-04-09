/**
 * Safe wrapper around Tauri's invoke API.
 *
 * When running inside the Tauri webview, this delegates to the real `invoke`.
 * When running in a plain browser (dev server without Tauri, Playwright, etc.),
 * it throws a descriptive error instead of crashing on missing IPC.
 *
 * All service modules should import `invoke` from here instead of
 * `@tauri-apps/api/core`.
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen, type UnlistenFn } from '@tauri-apps/api/event'

/** True when running inside the Tauri webview runtime. */
export const isTauri: boolean = Boolean(
  typeof window !== 'undefined' && (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__,
)

/**
 * Call a Tauri command. Returns the command result when inside Tauri.
 * When outside Tauri (plain browser / Playwright), returns a sensible
 * empty default so views render their empty states without error banners.
 */
export function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    // Return empty defaults so views show clean empty states
    // The return type T is typically an array, object, string, or void
    return Promise.resolve([] as unknown as T)
  }
  return tauriInvoke<T>(cmd, args)
}

export type { UnlistenFn }

/**
 * Listen for a Tauri event. Returns a no-op unlistener when outside Tauri.
 */
export function listen<T>(
  event: string,
  handler: (event: { payload: T }) => void,
): Promise<UnlistenFn> {
  if (!isTauri) {
    // Return a no-op unlistener — events can't fire outside Tauri anyway
    return Promise.resolve(() => {})
  }
  return tauriListen<T>(event, handler)
}
