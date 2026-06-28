(function () {
  if (window.top !== window.self) return;

  const tauri = window.__TAURI__;
  if (!tauri?.core?.invoke) return;

  const { invoke } = tauri.core;
  const listen = tauri.event?.listen;

  const EXIT_PATTERN = /choose your exit|leave player|exit player|back to browse|leave watch/i;
  const SETTINGS_ITEM_ID = "qstarem-settings-item";
  const RETURN_ITEM_ID = "qstarem-return-item";
  const SETTINGS_PANEL_ID = "qstarem-settings-panel";

  const SITE_MENU_LABELS = [
    /^account preferences$/i,
    /^appearance$/i,
    /^subtitles$/i,
  ];

  const ICON_OPTIONS = [
    { id: 1, label: "Q Play" },
    { id: 2, label: "Film Reel" },
    { id: 3, label: "Z Waves" },
    { id: 4, label: "Viewfinder" },
    { id: 5, label: "Orbital" },
    { id: 6, label: "Clapper" },
  ];

  let enhanceScheduled = false;
  let updateListenerBound = false;

  function normalizeText(value) {
    return (value || "").replace(/\s+/g, " ").trim();
  }

  function iconSrc(id) {
    const file = id === 1 ? "icons/icon.png" : `icons/variants/${id}.png`;
    if (typeof tauri.core.convertFileSrc === "function") {
      return tauri.core.convertFileSrc(file);
    }
    return file;
  }

  function matchesSiteMenuLabel(text) {
    return SITE_MENU_LABELS.some((pattern) => pattern.test(normalizeText(text)));
  }

  function matchesExitControl(node) {
    if (!node || node.nodeType !== 1) return false;
    if (node.closest(`#${RETURN_ITEM_ID}, #${SETTINGS_ITEM_ID}, #${SETTINGS_PANEL_ID}`)) {
      return false;
    }
    const text = normalizeText(node.textContent);
    const aria = normalizeText(node.getAttribute("aria-label"));
    const title = normalizeText(node.getAttribute("title"));
    const combined = `${text} ${aria} ${title}`;
    if (!combined) return false;
    if (EXIT_PATTERN.test(combined)) return true;
    if (/^exit$/i.test(text) && text.length < 24) return true;
    return false;
  }

  function findExitControls(root) {
    const matches = [];
    root.querySelectorAll("button, a, [role='button'], [role='link']").forEach((node) => {
      if (matchesExitControl(node)) matches.push(node);
    });
    return matches;
  }

  function clickOriginalExit() {
    const hidden = document.querySelector(".qstarem-hidden-exit");
    if (hidden) {
      hidden.click();
      return true;
    }
    const exit = findExitControls(document)[0];
    if (exit) {
      exit.click();
      return true;
    }
    return false;
  }

  function requestExitPlayer() {
    closeSettingsPanel();
    if (clickOriginalExit()) return;
    if (window.history.length > 1) {
      window.history.back();
    }
  }

  function isVisible(node) {
    if (!node || !node.isConnected) return false;
    const style = window.getComputedStyle(node);
    if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) {
      return false;
    }
    const rect = node.getBoundingClientRect();
    return rect.width > 8 && rect.height > 8;
  }

  function findSiteMenuRow(labelPattern) {
    const candidates = document.querySelectorAll(
      "a, button, [role='button'], [role='menuitem'], li, div, span, p",
    );

    for (const node of candidates) {
      if (!isVisible(node)) continue;
      const text = normalizeText(node.textContent);
      if (!labelPattern.test(text)) continue;
      if (text.length > 48) continue;
      return node.closest("a, button, [role='button'], li, div") || node;
    }

    return null;
  }

  function findSiteMenuContainer() {
    const anchors = SITE_MENU_LABELS.map((pattern) => findSiteMenuRow(pattern)).filter(Boolean);
    if (anchors.length === 0) return null;

    const first = anchors[0];
    let parent = first.parentElement;
    while (parent && parent !== document.body) {
      const matches = Array.from(parent.children).filter((child) => {
        const text = normalizeText(child.textContent);
        return SITE_MENU_LABELS.some((pattern) => pattern.test(text));
      });
      if (matches.length >= 2) {
        return { container: parent, template: matches[matches.length - 1] };
      }
      parent = parent.parentElement;
    }

    return { container: first.parentElement, template: first };
  }

  function cloneMenuRow(template, label, id, onClick) {
    const row = template.cloneNode(true);
    row.id = id;
    row.querySelectorAll("[id]").forEach((node) => node.removeAttribute("id"));

    const textNodes = [];
    const walker = document.createTreeWalker(row, NodeFilter.SHOW_TEXT);
    let current = walker.nextNode();
    while (current) {
      if (normalizeText(current.textContent)) textNodes.push(current);
      current = walker.nextNode();
    }

    if (textNodes.length > 0) {
      textNodes[0].textContent = label;
      for (let index = 1; index < textNodes.length; index += 1) {
        textNodes[index].textContent = "";
      }
    } else {
      const target = row.querySelector("span, p, a, button") || row;
      target.textContent = label;
    }

    const link = row.querySelector("a, button");
    if (link && link.tagName === "A") {
      link.setAttribute("href", "#");
    }

    row.addEventListener(
      "click",
      (event) => {
        event.preventDefault();
        event.stopPropagation();
        onClick();
      },
      true,
    );

    return row;
  }

  function injectMenuItems() {
    const menu = findSiteMenuContainer();
    if (!menu) return;

    const { container, template } = menu;

    if (!container.querySelector(`#${SETTINGS_ITEM_ID}`)) {
      container.appendChild(
        cloneMenuRow(template, "App settings", SETTINGS_ITEM_ID, () => {
          openSettingsPanel();
        }),
      );
    }

    if (document.querySelector("video") && !container.querySelector(`#${RETURN_ITEM_ID}`)) {
      container.insertBefore(
        cloneMenuRow(template, "Return to browse", RETURN_ITEM_ID, () => {
          requestExitPlayer();
        }),
        container.firstChild,
      );
    }
  }

  function renderIconChoices(selectedId) {
    ensureSettingsPanel();
    const grid = document.getElementById("qstarem-icon-choices");
    if (!grid) return;

    grid.innerHTML = "";
    ICON_OPTIONS.forEach((option) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "qstarem-icon-choice";
      button.dataset.iconId = String(option.id);
      button.setAttribute("aria-pressed", option.id === selectedId ? "true" : "false");

      const image = document.createElement("img");
      image.src = iconSrc(option.id);
      image.alt = option.label;
      image.width = 56;
      image.height = 56;

      const label = document.createElement("span");
      label.textContent = option.label;

      button.append(image, label);
      button.addEventListener("click", () => {
        grid.dataset.selectedIconId = String(option.id);
        renderIconChoices(option.id);
      });

      grid.appendChild(button);
    });
    grid.dataset.selectedIconId = String(selectedId);
  }

  function selectedIconIdFromPanel() {
    const grid = document.getElementById("qstarem-icon-choices");
    const value = Number(grid?.dataset.selectedIconId || 1);
    if (Number.isNaN(value)) return 1;
    return Math.min(6, Math.max(1, value));
  }

  function ensureSettingsPanel() {
    if (document.getElementById(SETTINGS_PANEL_ID)) return;

    const panel = document.createElement("div");
    panel.id = SETTINGS_PANEL_ID;
    panel.hidden = true;
    panel.innerHTML = `
      <div class="qstarem-settings-backdrop" data-qstarem-close></div>
      <section class="qstarem-settings-sheet" role="dialog" aria-label="App settings">
        <header class="qstarem-settings-header">
          <h2>App settings</h2>
          <button type="button" class="qstarem-settings-close" data-qstarem-close aria-label="Close">×</button>
        </header>
        <div class="qstarem-settings-body">
          <label class="qstarem-settings-field">
            <span>Home URL</span>
            <input id="qstarem-home-url" type="url" autocomplete="off" />
          </label>
          <button type="button" class="qstarem-settings-link" id="qstarem-reset-home">Reset to zstream.mov</button>
          <fieldset class="qstarem-settings-field">
            <legend>Ad blocker</legend>
            <label><input type="radio" name="qstarem-blocker" value="ublock" /> uBlock Origin</label>
            <label><input type="radio" name="qstarem-blocker" value="adguard" /> AdGuard</label>
            <label><input type="radio" name="qstarem-blocker" value="none" /> Off</label>
          </fieldset>
          <label class="qstarem-settings-toggle">
            <span>P-Stream userscript</span>
            <input id="qstarem-pstream" type="checkbox" />
          </label>
          <div class="qstarem-settings-field">
            <span>App icon</span>
            <div id="qstarem-icon-choices" class="qstarem-icon-grid"></div>
          </div>
          <div class="qstarem-settings-field qstarem-update-section">
            <span>Updates</span>
            <p id="qstarem-app-version" class="qstarem-update-version">QStarem</p>
            <p id="qstarem-update-status" class="qstarem-update-status">Tap below to check for updates.</p>
            <button type="button" class="qstarem-settings-link" id="qstarem-check-updates">Check for updates</button>
            <button type="button" class="qstarem-settings-primary qstarem-update-install" id="qstarem-install-update" hidden>Install update</button>
          </div>
        </div>
        <footer class="qstarem-settings-footer">
          <button type="button" class="qstarem-settings-primary" id="qstarem-save-settings">Save</button>
          <button type="button" class="qstarem-settings-secondary" id="qstarem-open-settings-window">Open settings window</button>
          <button type="button" class="qstarem-settings-secondary" id="qstarem-clear-data">Clear browsing data</button>
        </footer>
      </section>
    `;
    document.body.appendChild(panel);

    panel.querySelectorAll("[data-qstarem-close]").forEach((node) => {
      node.addEventListener("click", closeSettingsPanel);
    });
    panel.querySelector("#qstarem-reset-home").addEventListener("click", () => {
      panel.querySelector("#qstarem-home-url").value = "https://zstream.mov";
    });
    panel.querySelector("#qstarem-save-settings").addEventListener("click", saveSettingsFromPanel);
    panel.querySelector("#qstarem-open-settings-window").addEventListener("click", () => {
      closeSettingsPanel();
      invoke("open_settings").catch((error) => {
        console.error("[QStarem] open_settings failed", error);
      });
    });
    panel.querySelector("#qstarem-clear-data").addEventListener("click", async () => {
      closeSettingsPanel();
      try {
        await invoke("clear_browsing_data");
      } catch (error) {
        console.error("[QStarem] clear_browsing_data failed", error);
      }
    });
    panel.querySelector("#qstarem-check-updates").addEventListener("click", async () => {
      try {
        const status = await invoke("check_for_updates");
        updateStatusFromPayload(status);
      } catch (error) {
        updateStatusFromPayload({ phase: "error", message: String(error) });
      }
    });
    panel.querySelector("#qstarem-install-update").addEventListener("click", async () => {
      closeSettingsPanel();
      try {
        await invoke("install_pending_update");
      } catch (error) {
        console.error("[QStarem] install_pending_update failed", error);
      }
    });

    bindUpdateListener();
  }

  function updateStatusFromPayload(status) {
    const panel = document.getElementById(SETTINGS_PANEL_ID);
    if (!panel) return;

    const statusEl = panel.querySelector("#qstarem-update-status");
    const installBtn = panel.querySelector("#qstarem-install-update");
    if (!statusEl || !installBtn) return;

    const phase = status.phase || "idle";
    const progress = Number(status.progress || 0);
    let message = status.message || "Up to date.";

    if (phase === "downloading" && progress > 0) {
      message = `${message} ${Math.round(progress * 100)}%`;
    } else if (phase === "checking") {
      message = status.message || "Checking for updates…";
    } else if (phase === "ready") {
      message = status.message || "Update ready to install.";
    }

    statusEl.textContent = message;
    installBtn.hidden = phase !== "ready";
  }

  async function populateSettingsPanel() {
    ensureSettingsPanel();
    const panel = document.getElementById(SETTINGS_PANEL_ID);

    try {
      const settings = await invoke("get_settings");
      panel.querySelector("#qstarem-home-url").value = settings.home_url || "https://zstream.mov";
      panel.querySelector("#qstarem-pstream").checked = settings.p_stream_enabled !== false;
      const blocker = settings.ad_blocker || "ublock";
      panel.querySelectorAll('input[name="qstarem-blocker"]').forEach((input) => {
        input.checked = input.value === blocker;
      });
      renderIconChoices(settings.app_icon_id || 1);
    } catch (error) {
      console.error("[QStarem] get_settings failed", error);
    }

    try {
      const version = await invoke("get_app_version");
      const versionEl = panel.querySelector("#qstarem-app-version");
      if (versionEl) {
        versionEl.textContent = `Installed: QStarem v${version}`;
      }
    } catch (_error) {
      /* ignore */
    }

    try {
      const status = await invoke("get_update_status");
      updateStatusFromPayload(status);
    } catch (_error) {
      /* ignore */
    }
  }

  function bindUpdateListener() {
    if (updateListenerBound || typeof listen !== "function") return;
    updateListenerBound = true;
    listen("update-status-changed", (event) => {
      updateStatusFromPayload(event.payload || {});
    }).catch((error) => {
      console.error("[QStarem] update listener failed", error);
      updateListenerBound = false;
    });
  }

  function openSettingsPanel() {
    populateSettingsPanel();
    const panel = document.getElementById(SETTINGS_PANEL_ID);
    panel.hidden = false;
    document.body.classList.add("qstarem-settings-open");
  }

  function closeSettingsPanel() {
    const panel = document.getElementById(SETTINGS_PANEL_ID);
    if (!panel) return;
    panel.hidden = true;
    document.body.classList.remove("qstarem-settings-open");
  }

  async function saveSettingsFromPanel() {
    const panel = document.getElementById(SETTINGS_PANEL_ID);
    const homeUrl = panel.querySelector("#qstarem-home-url").value.trim() || "https://zstream.mov";
    const adBlocker =
      panel.querySelector('input[name="qstarem-blocker"]:checked')?.value || "ublock";
    const pStreamEnabled = panel.querySelector("#qstarem-pstream").checked;
    const appIconId = selectedIconIdFromPanel();

    try {
      await invoke("save_settings", {
        settings: {
          home_url: homeUrl,
          ad_blocker: adBlocker,
          p_stream_enabled: pStreamEnabled,
          app_icon_id: appIconId,
        },
      });
      closeSettingsPanel();
    } catch (error) {
      console.error("[QStarem] save_settings failed", error);
    }
  }

  function hideExitControls() {
    if (!document.querySelector("video")) return;
    findExitControls(document).forEach((node) => {
      if (node.classList.contains("qstarem-hidden-exit")) return;
      node.classList.add("qstarem-hidden-exit");
    });
  }

  function scheduleEnhance() {
    if (enhanceScheduled) return;
    enhanceScheduled = true;
    requestAnimationFrame(() => {
      enhanceScheduled = false;
      hideExitControls();
      injectMenuItems();
    });
  }

  function boot() {
    ensureSettingsPanel();
    hideExitControls();
    scheduleEnhance();

    const observer = new MutationObserver(() => {
      scheduleEnhance();
    });
    observer.observe(document.documentElement, { childList: true, subtree: true });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }

  window.addEventListener("pageshow", () => {
    hideExitControls();
    scheduleEnhance();
  });
})();
