const { invoke } = window.__TAURI__.core;

const DEFAULT_HOME_URL = "https://zstream.mov";

const homeUrl = document.getElementById("homeUrl");
const resetUrl = document.getElementById("resetUrl");
const pStreamEnabled = document.getElementById("pStreamEnabled");
const saveButton = document.getElementById("save");
const clearDataButton = document.getElementById("clearData");

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

async function loadSettings() {
  const settings = await invoke("get_settings");
  homeUrl.value = settings.home_url;
  pStreamEnabled.checked = settings.p_stream_enabled;
  setAdBlocker(settings.ad_blocker);
}

async function saveSettings() {
  await invoke("save_settings", {
    settings: {
      home_url: homeUrl.value.trim() || DEFAULT_HOME_URL,
      ad_blocker: selectedAdBlocker(),
      p_stream_enabled: pStreamEnabled.checked,
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
