"use client";

import { useEffect } from "react";

/**
 * React does not execute <script dangerouslySetInnerHTML> tags.
 * Append real script elements so browser-run IIFEs can wire DOM listeners.
 */
export function useDomScript(script: string): void {
  useEffect(() => {
    if (!script.trim()) {
      return;
    }
    const run = () => {
      const scriptEl = document.createElement("script");
      scriptEl.textContent = script;
      document.body.appendChild(scriptEl);
      return scriptEl;
    };
    const scriptEl = run();
    return () => {
      scriptEl.remove();
    };
  }, [script]);
}

export function ClientScript({ script }: { script: string }): null {
  useDomScript(script);
  return null;
}

/** Injects JSON blobs that other panel scripts read via querySelector(...).textContent */
export function DomJsonScript({
  marker,
  json,
}: {
  marker: string;
  json: string;
}): null {
  useEffect(() => {
    if (!json) {
      return;
    }
    const selector = `script[type="application/json"][${marker}]`;
    if (document.querySelector(selector)) {
      return;
    }
    const el = document.createElement("script");
    el.type = "application/json";
    el.setAttribute(marker, "true");
    el.textContent = json;
    document.body.appendChild(el);
    return () => {
      el.remove();
    };
  }, [marker, json]);
  return null;
}