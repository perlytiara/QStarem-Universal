(function () {
  const EXIT_PATTERN = /choose your exit|leave player|exit player|back to browse|leave watch/i;
  const STYLE_ID = "qstarem-shell-styles";

  function injectStyles() {
    if (document.getElementById(STYLE_ID)) return;
    const css = document.createElement("style");
    css.id = STYLE_ID;
    css.textContent = `/* injected via QStarem shell */`;
    document.head.appendChild(css);
  }

  function matchesExitControl(node) {
    if (!node || node.nodeType !== 1) return false;
    const text = (node.textContent || "").trim();
    const aria = (node.getAttribute("aria-label") || "").trim();
    const title = (node.getAttribute("title") || "").trim();
    const combined = `${text} ${aria} ${title}`;
    if (!combined) return false;
    if (EXIT_PATTERN.test(combined)) return true;
    if (/^exit$/i.test(text) && text.length < 24) return true;
    return false;
  }

  function findExitControls(root) {
    const candidates = root.querySelectorAll(
      "button, a, [role='button'], [role='link']"
    );
    const matches = [];
    candidates.forEach((node) => {
      if (matchesExitControl(node)) matches.push(node);
    });
    return matches;
  }

  function enhanceExitControls() {
    injectStyles();
    findExitControls(document).forEach((node) => {
      if (node.classList.contains("qstarem-embedded-exit")) return;
      node.classList.add("qstarem-embedded-exit");
      if (!node.getAttribute("title")) {
        node.setAttribute("title", "Return from player (Esc)");
      }
    });
  }

  function boot() {
    enhanceExitControls();
    const observer = new MutationObserver(() => enhanceExitControls());
    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }

  window.addEventListener("load", enhanceExitControls);
  window.addEventListener("pageshow", enhanceExitControls);
})();
