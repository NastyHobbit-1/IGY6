export const PRODUCT_NAV_SCRIPT = `
(() => {
  const root = document.querySelector("[data-product-app]");
  if (!root || root.getAttribute("data-product-nav-wired") === "true") return;
  root.setAttribute("data-product-nav-wired", "true");

  const MODE_KEY = "igy6-ui-mode";
  const sections = Array.from(root.querySelectorAll("[data-product-section]"));
  const navButtons = Array.from(root.querySelectorAll("[data-product-nav]"));
  const legacyTabs = {
    command: "tab-results",
    add: "tab-add-data",
    evidence: "tab-results",
    gallery: "tab-results",
    activity: "tab-work",
    system: "tab-advanced",
    settings: "tab-settings"
  };

  const postJson = async (path, body) => {
    const response = await fetch(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body || {}),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };

  const showSection = (sectionId, scrollTarget) => {
    sections.forEach((section) => {
      const active = section.getAttribute("data-product-section") === sectionId;
      section.hidden = !active;
      if (active) section.classList.add("productSectionActive");
      else section.classList.remove("productSectionActive");
    });
    navButtons.forEach((button) => {
      const active = button.getAttribute("data-product-nav") === sectionId;
      button.setAttribute("aria-current", active ? "page" : "false");
      button.classList.toggle("productNavActive", active);
    });
    const legacyId = legacyTabs[sectionId];
    if (legacyId) {
      const tab = document.getElementById(legacyId);
      if (tab) tab.checked = true;
    }
    if (scrollTarget) {
      requestAnimationFrame(() => {
        document.getElementById(scrollTarget)?.scrollIntoView({ behavior: "smooth", block: "start" });
      });
    } else {
      root.querySelector("[data-product-main]")?.scrollTo({ top: 0, behavior: "smooth" });
    }
  };

  window.igy6SwitchSection = showSection;
  window.igy6Navigate = (sectionId, scrollTarget) => showSection(sectionId, scrollTarget);

  const applyUiMode = (mode) => {
    const normalized = mode === "full" ? "full" : "simple";
    root.setAttribute("data-ui-mode", normalized);
    document.body.classList.toggle("igy6-full-mode", normalized === "full");
    document.body.classList.toggle("igy6-simple-mode", normalized === "simple");
    root.querySelectorAll("[data-ui-mode]").forEach((button) => {
      const active = button.getAttribute("data-ui-mode") === normalized;
      button.classList.toggle("productModeActive", active);
      button.setAttribute("aria-pressed", active ? "true" : "false");
    });
    try {
      window.localStorage.setItem(MODE_KEY, normalized);
    } catch {
      /* ignore storage errors */
    }
  };

  const storedMode = (() => {
    try {
      return window.localStorage.getItem(MODE_KEY);
    } catch {
      return null;
    }
  })();
  applyUiMode(storedMode === "full" ? "full" : "simple");

  root.querySelectorAll("[data-ui-mode]").forEach((button) => {
    button.addEventListener("click", () => {
      applyUiMode(button.getAttribute("data-ui-mode"));
    });
  });

  navButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const sectionId = button.getAttribute("data-product-nav");
      if (sectionId) showSection(sectionId, null);
    });
  });

  root.querySelectorAll("[data-product-go]").forEach((element) => {
    element.addEventListener("click", (event) => {
      const target = event.currentTarget;
      const sectionId = target.getAttribute("data-product-go");
      const scrollTarget = target.getAttribute("data-product-scroll");
      if (sectionId) showSection(sectionId, scrollTarget || null);
    });
  });

  const retryWorkItem = async (workItemId, button) => {
    if (!workItemId) return;
    if (button) button.disabled = true;
    try {
      await postJson("/api/work-items/" + encodeURIComponent(workItemId) + "/status", {
        status: "queued",
        actor_id: "local-owner",
      });
      window.setTimeout(() => window.location.reload(), 900);
    } catch (error) {
      window.alert(error instanceof Error ? error.message : String(error));
      if (button) button.disabled = false;
    }
  };

  root.querySelectorAll("[data-retry-failed-work]").forEach((button) => {
    button.addEventListener("click", () => {
      retryWorkItem(button.getAttribute("data-work-item-id"), button);
    });
  });

  const ensureSearchIndex = async (options = {}) => {
    const { reloadOnSuccess = true, silent = false } = options;
    const statusEl = root.querySelector("[data-search-status-label]");
    if (statusEl && !silent) statusEl.textContent = "Search is being prepared";
    try {
      await postJson("/api/memory/vector/chunks/ensure", {});
      root.setAttribute("data-search-state", "ok");
      if (statusEl) statusEl.textContent = "Search is ready";
      if (reloadOnSuccess) window.setTimeout(() => window.location.reload(), 700);
      return true;
    } catch (error) {
      root.setAttribute("data-search-state", "warning");
      if (statusEl) statusEl.textContent = "Search needs attention";
      if (!silent) window.alert(error instanceof Error ? error.message : String(error));
      return false;
    }
  };

  root.querySelectorAll("[data-product-vector-ensure]").forEach((button) => {
    button.addEventListener("click", async () => {
      button.disabled = true;
      await ensureSearchIndex({ reloadOnSuccess: true, silent: false });
      button.disabled = false;
    });
  });

  const autoEnsureSearch = async () => {
    const state = root.getAttribute("data-search-state");
    const reachable = root.getAttribute("data-search-reachable") === "true";
    if (!reachable || state === "ok" || root.getAttribute("data-search-auto-tried") === "true") return;
    root.setAttribute("data-search-auto-tried", "true");
    await ensureSearchIndex({ reloadOnSuccess: true, silent: true });
  };

  const attentionDrawer = root.querySelector("[data-attention-drawer]");
  const toggleAttention = (open) => {
    if (!attentionDrawer) return;
    attentionDrawer.hidden = !open;
  };
  root.querySelector("[data-attention-toggle]")?.addEventListener("click", () => {
    toggleAttention(attentionDrawer?.hidden !== false);
  });
  root.querySelector("[data-attention-close]")?.addEventListener("click", () => toggleAttention(false));

  root.querySelectorAll("[data-attention-action]").forEach((button) => {
    button.addEventListener("click", async () => {
      const sectionId = button.getAttribute("data-attention-section");
      const scrollTarget = button.getAttribute("data-attention-scroll");
      const retryId = button.getAttribute("data-retry-work-item-id");
      toggleAttention(false);
      if (retryId) {
        await retryWorkItem(retryId, button);
        return;
      }
      if (sectionId === "system" && button.textContent?.toLowerCase().includes("repair")) {
        await ensureSearchIndex({ reloadOnSuccess: true, silent: false });
        return;
      }
      if (sectionId) showSection(sectionId, scrollTarget || null);
    });
  });

  const gallery = root.querySelector("[data-image-gallery]");
  if (gallery) {
    const items = Array.from(gallery.querySelectorAll("[data-gallery-item]"));
    const grid = gallery.querySelector("[data-gallery-grid]");
    const countEl = gallery.querySelector("[data-gallery-count]");
    const searchInput = gallery.querySelector("[data-gallery-search]");
    const sortSelect = gallery.querySelector("[data-gallery-sort]");
    const typeFilter = gallery.querySelector("[data-gallery-filter-type]");
    const sourceFilter = gallery.querySelector("[data-gallery-filter-source]");

    gallery.querySelectorAll("[data-gallery-thumb]").forEach((img) => {
      img.addEventListener("error", () => {
        img.style.display = "none";
        const placeholder = img.parentElement?.querySelector("[data-gallery-placeholder]");
        if (placeholder instanceof HTMLElement) placeholder.hidden = false;
      });
    });

    const applyGalleryFilters = () => {
      const query = (searchInput?.value || "").trim().toLowerCase();
      const typeValue = typeFilter?.value || "all";
      const sourceValue = sourceFilter?.value || "all";
      const sortValue = sortSelect?.value || "newest";
      const visible = items.filter((item) => {
        const title = item.getAttribute("data-title") || "";
        const source = item.getAttribute("data-source") || "";
        const extension = item.getAttribute("data-extension") || "";
        const haystack = title + " " + source;
        if (query && !haystack.includes(query)) return false;
        if (typeValue !== "all") {
          const normalized = typeValue === "jpg" ? ["jpg", "jpeg"] : [typeValue];
          if (!normalized.includes(extension)) return false;
        }
        if (sourceValue !== "all" && source !== sourceValue.toLowerCase()) return false;
        return true;
      });
      visible.sort((left, right) => {
        if (sortValue === "source") {
          return (left.getAttribute("data-source") || "").localeCompare(right.getAttribute("data-source") || "");
        }
        if (sortValue === "type") {
          return (left.getAttribute("data-extension") || "").localeCompare(right.getAttribute("data-extension") || "");
        }
        if (sortValue === "status") {
          return (left.getAttribute("data-status") || "").localeCompare(right.getAttribute("data-status") || "");
        }
        if (sortValue === "size") {
          return Number(right.getAttribute("data-size-bytes") || 0) - Number(left.getAttribute("data-size-bytes") || 0);
        }
        const leftTs = Date.parse(left.getAttribute("data-timestamp") || "") || 0;
        const rightTs = Date.parse(right.getAttribute("data-timestamp") || "") || 0;
        return sortValue === "oldest" ? leftTs - rightTs : rightTs - leftTs;
      });
      items.forEach((item) => {
        item.style.display = "none";
      });
      visible.forEach((item) => {
        item.style.display = "";
        grid?.appendChild(item);
      });
      if (countEl) {
        countEl.textContent = "Showing " + visible.length + " image" + (visible.length === 1 ? "" : "s");
      }
    };

    searchInput?.addEventListener("input", applyGalleryFilters);
    sortSelect?.addEventListener("change", applyGalleryFilters);
    typeFilter?.addEventListener("change", applyGalleryFilters);
    sourceFilter?.addEventListener("change", applyGalleryFilters);
    applyGalleryFilters();
  }

  showSection("command", null);
  void autoEnsureSearch();
})();
`;