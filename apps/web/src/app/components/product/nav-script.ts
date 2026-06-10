export const PRODUCT_NAV_SCRIPT = `
(() => {
  const root = document.querySelector("[data-product-app]");
  if (!root || root.getAttribute("data-product-nav-wired") === "true") return;
  root.setAttribute("data-product-nav-wired", "true");

  const sections = Array.from(root.querySelectorAll("[data-product-section]"));
  const navButtons = Array.from(root.querySelectorAll("[data-product-nav]"));
  const legacyTabs = {
    command: "tab-results",
    add: "tab-add-data",
    evidence: "tab-results",
    activity: "tab-work",
    system: "tab-advanced",
    settings: "tab-settings"
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

  showSection("command", null);
})();
`;