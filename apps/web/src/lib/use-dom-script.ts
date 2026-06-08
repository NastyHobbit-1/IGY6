import { useEffect } from "react";

/**
 * React does not execute <script dangerouslySetInnerHTML> tags.
 * Append a real script element so browser-run IIFEs can wire DOM listeners.
 */
export function useDomScript(script: string): void {
  useEffect(() => {
    if (!script.trim()) {
      return;
    }
    const scriptEl = document.createElement("script");
    scriptEl.textContent = script;
    document.body.appendChild(scriptEl);
    return () => {
      scriptEl.remove();
    };
    // Wire once per component mount; IIFE guards prevent duplicate listeners.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}