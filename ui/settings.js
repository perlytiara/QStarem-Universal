const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const DEFAULT_HOME_URL = "https://zstream.mov";

const homeUrl = document.getElementById("homeUrl");
const resetUrl = document.getElementById("resetUrl");
const pStreamEnabled = document.getElementById("pStreamEnabled");
const iconChoices = document.getElementById("iconChoices");
const saveButton = document.getElementById("save");
const clearDataButton = document.getElementById("clearData");
const updateStatus = document.getElementById("updateStatus");
const checkUpdatesButton = document.getElementById("checkUpdates");
const installUpdateButton = document.getElementById("installUpdate");

const ICON_OPTIONS = [
  { id: 1, label: "Q Play" },
  { id: 2, label: "Film Reel" },
  { id: 3, label: "Z Waves" },
  { id: 4, label: "Viewfinder" },
  { id: 5, label: "Orbital" },
  { id: 6, label: "Clapper" },
];

let selectedIconId = 1;

function selectedAdBlocker() {
  const selected = document.querySelector('input[name="adBlocker"]:checked');
  return selected ? selected.value : "ublock";
}

function setAdBlocker(value) {
  const input = document.querySelector(`input[name="adBlocker"][value="${value}"]`);
  if (input) {
    input.checked = true;
  }
}

function renderIconChoices() {
  iconChoices.innerHTML = "";

  ICON_OPTIONS.forEach((option) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "icon-choice";
    button.dataset.iconId = String(option.id);
    button.setAttribute("aria-pressed", option.id === selectedIconId ? "true" : "false");

    const image = document.createElement("img");
    image.src = `icons/icon-${option.id}.png`;
    image.alt = option.label;
    image.width = 64;
    image.height = 64;

    const label = document.createElement("span");
    label.textContent = option.label;

    button.append(image, label);
    button.addEventListener("click", () => {
      selectedIconId = option.id;
      renderIconChoices();
    });

    iconChoices.appendChild(button);
  });
}

function renderUpdateStatus(status) {
  const version = status.current_version || "unknown";
  let message = status.message || `Current version: v${version}`;

  if (status.phase === "downloading" && status.progress > 0) {
    message = `${message} ${Math.round(status.progress * 100)}%`;
  }

  updateStatus.textContent = message;
  installUpdateButton.hidden = status.phase !== "ready";
  checkUpdatesButton.disabled =
    status.phase === "checking" ||
    status.phase === "downloading" ||
    status.phase === "installing";
  installUpdateButton.disabled = status.phase === "installing";
  if (status.phase === "installing") {
    installUpdateButton.textContent = "Installing…";
    installUpdateButton.hidden = false;
  } else {
    installUpdateButton.textContent = "Install and restart";
  }
}

async function refreshUpdateStatus() {
  const status = await invoke("get_update_status");
  renderUpdateStatus(status);
}

async function loadSettings() {
  const settings = await invoke("get_settings");
  homeUrl.value = settings.home_url;
  pStreamEnabled.checked = settings.p_stream_enabled;
  selectedIconId = settings.app_icon_id || 1;
  setAdBlocker(settings.ad_blocker);
  renderIconChoices();
  await refreshUpdateStatus();
}

async function saveSettings() {
  await invoke("save_settings", {
    settings: {
      home_url: homeUrl.value.trim() || DEFAULT_HOME_URL,
      ad_blocker: selectedAdBlocker(),
      p_stream_enabled: pStreamEnabled.checked,
      app_icon_id: selectedIconId,
    },
  });
}

resetUrl.addEventListener("click", () => {
  homeUrl.value = DEFAULT_HOME_URL;
});

saveButton.addEventListener("click", async () => {
  await saveSettings();
});

clearDataButton.addEventListener("click", async () => {
  await invoke("clear_browsing_data");
});

checkUpdatesButton.addEventListener("click", async () => {
  try {
    const status = await invoke("check_for_updates");
    renderUpdateStatus(status);
  } catch (error) {
    updateStatus.textContent = `Update check failed: ${error}`;
  }
});

installUpdateButton.addEventListener("click", async () => {
  installUpdateButton.disabled = true;
  installUpdateButton.textContent = "Installing…";
  updateStatus.textContent = "Installing update. QStarem will restart when ready.";
  try {
    await invoke("install_pending_update");
  } catch (error) {
    installUpdateButton.disabled = false;
    installUpdateButton.textContent = "Install and restart";
    updateStatus.textContent = `Install failed: ${error}`;
  }
});

listen("update-status-changed", (event) => {
  renderUpdateStatus(event.payload);
}).catch((error) => {
  console.error("Failed to listen for update status", error);
});

async function loadFooterVersion() {
  const footer = document.getElementById("appFooter");
  if (!footer) return;
  const version = await invoke("get_app_version");
  footer.textContent = `QStarem ${version} · Rust/Tauri · Z-Stream · P-Stream`;
}

loadSettings().catch((error) => {
  console.error("Failed to load settings", error);
});

loadFooterVersion().catch((error) => {
  console.error("Failed to load app version", error);
});
