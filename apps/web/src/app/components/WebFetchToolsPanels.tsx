import { StatusPill } from "./ui/StatusPill";

export function WebFetchToolsPanels() {
  return (
    <>
      <section className="panelInset maxReachUrlFetch" data-max-reach-url-fetch aria-label="Automated deep fetch">
        <div className="subHeader">
          <h3>Automated deep fetch</h3>
          <StatusPill state="local-first" />
        </div>
        <p className="actionHint">
          Strongest built-in tier. Automatically starts host bridge and Playwright on your PC, then runs authorized collection techniques (headed/CDP Playwright, multi-profile passes, scroll/expand, session header where you provide it). Use for account pages you are authorized to access.
        </p>
        <label>
          <span>Page URL</span>
          <input name="max_reach_page_url" type="url" placeholder="https://example.com/article" />
        </label>
        <label>
          <span>How many link levels to follow</span>
          <select name="max_reach_page_depth" defaultValue="1">
            <option value="0">This page only</option>
            <option value="1">This page plus direct links</option>
            <option value="2">Two levels of links</option>
          </select>
        </label>
        <div className="guidedManualActions">
          <button type="button" data-max-reach-fetch-url>Automated deep fetch</button>
          <span>Can take up to 10 minutes. Optional env: MAX_REACH_HEADED=1, MAX_REACH_PROXY, MAX_REACH_CDP_PORT=9222.</span>
        </div>
        <div className="guidedManualResult" data-max-reach-url-result>
          <strong>Ready</strong>
          <span>Paste a URL above, then click Automated deep fetch — or say &quot;deep fetch https://...&quot; in Chat.</span>
        </div>
      </section>
      <section className="panelInset autoBypassUrlFetch" data-auto-bypass-url-fetch aria-label="Automated deep fetch">
        <div className="subHeader">
          <h3>Automated deep fetch</h3>
          <StatusPill state="local-first" />
        </div>
        <p className="actionHint">
          Paste any URL. No manual steps. IGY6 automatically runs authorized collection techniques, then uses session header from your local Chrome/Edge profiles (devtools-equivalent) where provided, launches Playwright against those sessions, and falls back to session-assisted fetch when needed. Data stays only inside this instance.
        </p>
        <label>
          <span>Page URL</span>
          <input name="auto_bypass_page_url" type="url" placeholder="https://example.com/article" />
        </label>
        <label>
          <span>How many link levels to follow</span>
          <select name="auto_bypass_page_depth" defaultValue="1">
            <option value="0">This page only</option>
            <option value="1">This page plus direct links</option>
            <option value="2">Two levels of links</option>
          </select>
        </label>
        <div className="guidedManualActions">
          <button type="button" data-auto-bypass-fetch-url>Automated deep fetch</button>
          <span>Runs full automatic collection with authorized techniques: session header, Playwright render, and session-assisted fetch fallback. Requires host bridge running locally for account pages.</span>
        </div>
        <div className="guidedManualResult" data-auto-bypass-url-result>
          <strong>Ready</strong>
          <span>Paste a URL above, then click Automated deep fetch — or say &quot;deep fetch https://...&quot; in Chat.</span>
        </div>
      </section>
      <section className="panelInset publicUrlFetch" data-public-url-fetch aria-label="Fetch public web page">
        <div className="subHeader">
          <h3>Fetch public web page</h3>
          <StatusPill state="local-first" />
        </div>
        <p className="actionHint">
          Paste a public page URL. No program login and no website sign-in required. Content is fetched once and stored only inside this IGY6 instance.
        </p>
        <label>
          <span>Page URL</span>
          <input name="public_page_url" type="url" placeholder="https://example.com/article" />
        </label>
        <label>
          <span>How many link levels to follow</span>
          <select name="public_page_depth" defaultValue="1">
            <option value="0">This page only</option>
            <option value="1">This page plus direct links</option>
            <option value="2">Two levels of links</option>
          </select>
        </label>
        <div className="guidedManualActions">
          <button type="button" data-fetch-public-url>Fetch page</button>
          <span>Works for public HTML pages. Account pages and heavy JavaScript sites may need automated deep or session-assisted fetch.</span>
        </div>
        <div className="guidedManualResult" data-public-url-result>
          <strong>Ready</strong>
          <span>Paste a URL above, then click Fetch page — or say &quot;fetch public https://...&quot; in Chat.</span>
        </div>
      </section>
      <section className="panelInset bypassUrlFetch" data-bypass-url-fetch aria-label="Session-assisted fetch (authorized session)">
        <div className="subHeader">
          <h3>Session-assisted fetch (authorized session)</h3>
          <StatusPill state="local-first" />
        </div>
        <p className="actionHint">
          Paste a URL plus a Cookie header or bearer token from a browser session you already own. Use this for account pages or signed-in pages you are authorized to access. Data stays only inside this IGY6 instance.
        </p>
        <label>
          <span>Page URL</span>
          <input name="bypass_page_url" type="url" placeholder="https://example.com/account/page" />
        </label>
        <label>
          <span>Cookie header (from browser dev tools)</span>
          <textarea name="bypass_cookie" rows={3} placeholder="session_id=...; auth=... (copy from Network tab Request Headers)" />
        </label>
        <label>
          <span>Bearer token (optional if cookie is set)</span>
          <input name="bypass_authorization" type="text" placeholder="eyJhbGciOi... or Bearer eyJhbGciOi..." />
        </label>
        <label>
          <span>How many link levels to follow</span>
          <select name="bypass_page_depth" defaultValue="1">
            <option value="0">This page only</option>
            <option value="1">This page plus direct links</option>
            <option value="2">Two levels of links</option>
          </select>
        </label>
        <div className="guidedManualActions">
          <button type="button" data-bypass-fetch-url>Session-assisted fetch</button>
          <span>Only use sessions you own. Cookies are sent once for this fetch and are not stored in the UI after submit.</span>
        </div>
        <div className="guidedManualResult" data-bypass-url-result>
          <strong>Ready</strong>
          <span>Paste a URL and session cookie or token, then click Session-assisted fetch.</span>
        </div>
      </section>
    </>
  );
}
