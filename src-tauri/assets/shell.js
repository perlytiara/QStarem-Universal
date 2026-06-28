(function () {
  const EXIT_PATTERN = /choose your exit|leave player|exit player|back to browse|leave watch/i;
  const STYLE_ID = "qstarem-shell-styles";
  const DRAG_ROOT_ID = "qstarem-drag-handles";
  const HEADER_DRAG_SELECTORS =
    "header, nav, [class*='header'], [class*='navbar'], [class*='top-bar']";

  function injectStyles() {
    if (document.getElementById(STYLE_ID)) return;
    const css = document.createElement("style");
    css.id = STYLE_ID;
    css.textContent = `/* injected via QStarem shell */`;
    document.head.appendChild(css);
  }

  function ensureEdgeDragHandles() {
    if (document.getElementById(DRAG_ROOT_ID)) return;

    const root = document.createElement("div");
    root.id = DRAG_ROOT_ID;
    root.innerHTML = `
      <div class="qstarem-drag-handle qstarem-drag-top" data-tauri-drag-region></div>
      <div class="qstarem-drag-handle qstarem-drag-left" data-tauri-drag-region></div>
      <div class="qstarem-drag-handle qstarem-drag-right" data-tauri-drag-region></div>
      <div class="qstarem-drag-handle qstarem-drag-bottom" data-tauri-drag-region></div>
    `;
    document.documentElement.appendChild(root);
  }

  function applyHeaderDragRegions() {
    document.querySelectorAll(HEADER_DRAG_SELECTORS).forEach((node) => {
      if (node.closest(`#${DRAG_ROOT_ID}`)) return;
      node.setAttribute("data-tauri-drag-region", "deep");
      node.classList.add("qstarem-drag-header");
    });
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
    injectStyles();
    ensureEdgeDragHandles();
    applyHeaderDragRegions();
    enhanceExitControls();
    const observer = new MutationObserver(() => {
      applyHeaderDragRegions();
      enhanceExitControls();
    });
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

  window.addEventListener("load", () => {
    applyHeaderDragRegions();
    enhanceExitControls();
  });
  window.addEventListener("pageshow", () => {
    applyHeaderDragRegions();
    enhanceExitControls();
  });
})();
