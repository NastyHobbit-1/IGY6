import { WEB_FETCH_MAX_REACH_SCRIPT, WEB_FETCH_AUTO_BYPASS_SCRIPT, WEB_FETCH_BYPASS_SCRIPT, WEB_FETCH_PUBLIC_SCRIPT } from "./constants";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { WebFetchToolsPanels } from "./WebFetchToolsPanels";

export function ChatWebFetchDock() {
  return (
    <section className="chatWebFetchDock" id="chat-web-fetch" data-chat-web-fetch aria-label="Web fetch tools">
      <details open>
        <summary>
          <strong>Web fetch tools</strong>
          <em>Deep fetch · Public fetch · Session fetch — also runnable from Chat</em>
        </summary>
        <p className="actionHint">
          Say <code>deep fetch https://example.com</code>, <code>session fetch https://...</code>, or <code>public fetch https://...</code> in Chat. Deep fetch starts host bridge and collection on your PC when needed.
        </p>
        <WebFetchToolsPanels />
      </details>
      <ClientScript script={WEB_FETCH_MAX_REACH_SCRIPT} />
      <ClientScript script={WEB_FETCH_AUTO_BYPASS_SCRIPT} />
      <ClientScript script={WEB_FETCH_PUBLIC_SCRIPT} />
      <ClientScript script={WEB_FETCH_BYPASS_SCRIPT} />
    </section>
  );
}

