import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function MinimalWorkspacePanel({
  sourceCount,
  evidenceCount,
  queuedWorkCount,
  pendingApprovals
}: {
  sourceCount: number;
  evidenceCount: number;
  queuedWorkCount: number;
  pendingApprovals: number;
}) {
  const script = `
(() => {
  const panel = document.querySelector("[data-minimal-workspace]");
  if (!panel || panel.getAttribute("data-wired") === "true") return;
  panel.setAttribute("data-wired", "true");
  const chatInput = document.querySelector("[data-chat-input]");
  const send = () => document.querySelector("[data-chat-send]")?.click();
  const fillAndSend = (text) => {
    if (chatInput) chatInput.value = text;
    send();
  };
  const switchTab = (tabId) => {
    const tab = document.getElementById(tabId);
    if (!tab) return;
    tab.checked = true;
    tab.dispatchEvent(new Event("change", { bubbles: true }));
  };
  panel.querySelector("[data-minimal-web-url]")?.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    const url = panel.querySelector("[data-minimal-web-url]")?.value?.trim() || "";
    if (!url) return;
    fillAndSend("get stuff from " + url);
  });
  panel.querySelector("[data-minimal-web-go]")?.addEventListener("click", () => {
    const url = panel.querySelector("[data-minimal-web-url]")?.value?.trim() || "";
    fillAndSend(url ? "get stuff from " + url : "I want to pull something from a website");
  });
  panel.querySelector("[data-minimal-add-text]")?.addEventListener("click", () => fillAndSend("I want to paste some text or notes"));
  panel.querySelector("[data-minimal-check-work]")?.addEventListener("click", () => fillAndSend("What's still running?"));
  panel.querySelector("[data-minimal-ask-evidence]")?.addEventListener("click", () => fillAndSend("What do you know so far?"));
  panel.querySelector("[data-minimal-settings]")?.addEventListener("click", () => switchTab("tab-settings"));
})();
`;
  return (
    <section className="minimalWorkspace responsivePanelGrid" data-minimal-workspace aria-label="Simple workspace">
      <article className="minimalSlot" data-minimal-slot="web">
        <h3>Pull from a website</h3>
        <p>Paste a link — I&apos;ll figure out whether you want public pages, a harder bypass, or the full-court press.</p>
        <div className="minimalSlotActions">
          <input type="url" data-minimal-web-url placeholder="https://example.com/page" />
          <button type="button" data-minimal-web-go>Go</button>
        </div>
      </article>
      <article className="minimalSlot" data-minimal-slot="add">
        <h3>Add your stuff</h3>
        <p>Notes, logs, exports, or anything you copied — paste it in through chat.</p>
        <button type="button" data-minimal-add-text>Paste text</button>
      </article>
      <article className="minimalSlot" data-minimal-slot="work">
        <h3>What&apos;s happening</h3>
        <p>{queuedWorkCount > 0 ? `${queuedWorkCount} thing(s) still in the queue.` : "Nothing waiting right now."}</p>
        <button type="button" data-minimal-check-work>Check on it</button>
      </article>
      <article className="minimalSlot" data-minimal-slot="results">
        <h3>What I saved</h3>
        <p>{evidenceCount > 0 ? `${evidenceCount} evidence piece(s) from ${sourceCount} source(s).` : "No saved answers yet — pull or add something first."}</p>
        <button type="button" data-minimal-ask-evidence>Ask about it</button>
      </article>
      <article className="minimalSlot" data-minimal-slot="settings">
        <h3>Safety &amp; settings</h3>
        <p>{pendingApprovals > 0 ? `${pendingApprovals} approval(s) need a look.` : "Password, approvals, and environment."}</p>
        <button type="button" data-minimal-settings>Open settings</button>
      </article>
      <ClientScript script={script} />
    </section>
  );
}

