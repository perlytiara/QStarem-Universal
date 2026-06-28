const { invoke } = window.__TAURI__.core;

const DEFAULT_HOME_URL = "https://zstream.mov";

const homeUrl = document.getElementById("homeUrl");
const resetUrl = document.getElementById("resetUrl");
const pStreamEnabled = document.getElementById("pStreamEnabled");
const iconChoices = document.getElementById("iconChoices");
const saveButton = document.getElementById("save");
const clearDataButton = document.getElementById("clearData");

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

async function loadSettings() {
  const settings = await invoke("get_settings");
  homeUrl.value = settings.home_url;
  pStreamEnabled.checked = settings.p_stream_enabled;
  selectedIconId = settings.app_icon_id || 1;
  setAdBlocker(settings.ad_blocker);
  renderIconChoices();
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

loadSettings().catch((error) => {
  console.error("Failed to load settings", error);
});
