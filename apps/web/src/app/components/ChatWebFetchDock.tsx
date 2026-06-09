import { WEB_FETCH_MAX_REACH_SCRIPT, WEB_FETCH_AUTO_BYPASS_SCRIPT, WEB_FETCH_BYPASS_SCRIPT, WEB_FETCH_PUBLIC_SCRIPT } from "./constants";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { WebFetchToolsPanels } from "./WebFetchToolsPanels";

export function ChatWebFetchDock() {
  return (
    <section className="chatWebFetchDock" id="chat-web-fetch" data-chat-web-fetch aria-label="Web fetch tools">
      <details open>
        <summary>
          <strong>Web fetch tools</strong>
          <em>Auto bypass · public fetch · session bypass — also runnable from Chat</em>
        </summary>
        <p className="actionHint">
          Say <code>max reach https://example.com</code>, <code>auto bypass https://...</code>, or <code>fetch public https://...</code> in Chat. Max reach auto-starts host bridge and Playwright on your PC.
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

