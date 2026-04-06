export const DEFAULT_SETTINGS = {
  serverUrl: import.meta.env.VITE_WEBSOCKET_SERVER || "ws://localhost:3001",
  gameMode: false
};

export function getSettings() {
  const saved = localStorage.getItem("netover_controller_settings");
  if (saved) {
    try {
      return { ...DEFAULT_SETTINGS, ...JSON.parse(saved) };
    } catch (e) {
      console.error(e);
    }
  }
  return DEFAULT_SETTINGS;
}

export function saveSettings(settings) {
  localStorage.setItem("netover_controller_settings", JSON.stringify(settings));
}
