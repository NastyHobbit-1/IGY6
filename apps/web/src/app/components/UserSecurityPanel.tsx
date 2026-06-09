import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";

export function UserSecurityPanel() {
  const script = `
(() => {
  const root = document.querySelector("[data-user-security]");
  if (!root || root.getAttribute("data-user-security-wired") === "true") return;
  root.setAttribute("data-user-security-wired", "true");

  root.querySelector("[data-user-change-password]")?.addEventListener("click", async () => {
    const current = document.getElementById("igy6-cur-password")?.value ?? "";
    const next = document.getElementById("igy6-new-password")?.value ?? "";
    const body = { current_password: current, new_password: next };
    const status = await (await fetch("/api/user/status")).json();
    if (status.totp_enabled) {
      const code = prompt("TOTP code?");
      if (code) body.totp_code = code;
    }
    const response = await fetch("/api/user/change-password", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    alert(await response.text());
  });

  root.querySelector("[data-user-link-totp]")?.addEventListener("click", async () => {
    const current = prompt("Current password to link TOTP:");
    if (!current) return;
    const response = await fetch("/api/user/generate-totp", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ current_password: current })
    });
    const payload = await response.json();
    if (!payload.secret) {
      alert(JSON.stringify(payload));
      return;
    }
    prompt("Add this secret to your authenticator app:", payload.secret + "\\n" + (payload.otpauth_url ?? ""));
    const code = prompt("Enter the 6-digit code from your app:");
    if (!code) return;
    const confirm = await fetch("/api/user/confirm-totp", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ current_password: current, totp_code: code })
    });
    alert(await confirm.text());
  });

  root.querySelector("[data-user-auth-status]")?.addEventListener("click", async () => {
    const status = await (await fetch("/api/user/status")).json();
    alert(JSON.stringify(status, null, 2));
  });
})();
`;

  return (
    <section className="panel settingsPanel tabContent" id="user-security" data-tab-panel="settings" data-user-security>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Settings</p>
          <h2>User &amp; Security</h2>
        </div>
        <StatusPill state="password-protected" />
      </div>
      <div className="guidedManualNotice">
        <strong>Local program password and optional TOTP.</strong>
        <span>Default password is <code>ThatDog123</code> until you change it. TOTP stays off until you link an authenticator app.</span>
      </div>
      <div className="settingsActions userSecurityActions">
        <label>
          <span>Current password</span>
          <input id="igy6-cur-password" type="password" autoComplete="current-password" />
        </label>
        <label>
          <span>New password</span>
          <input id="igy6-new-password" type="password" autoComplete="new-password" />
        </label>
        <button type="button" data-user-change-password>
          Change password
        </button>
        <button type="button" data-user-link-totp>
          Link authenticator (TOTP)
        </button>
        <button type="button" data-user-auth-status>
          Auth status
        </button>
      </div>
      <ClientScript script={script} />
    </section>
  );
}

