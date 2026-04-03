import { useState, useEffect } from "react";
import { getSettings, saveSettings } from "@/scripts/settings";
import CustomButton from "@/components/CustomButton";

export default function Config() {
  const [settings, setSettingsState] = useState({ serverUrl: "" });
  const [status, setStatus] = useState("");

  useEffect(() => {
    setSettingsState(getSettings());
  }, []);

  const handleSave = () => {
    saveSettings(settings);
    setStatus("Settings saved successfully!");
    setTimeout(() => setStatus(""), 3000);
  };

  return (
    <div className="max-w-2xl mx-auto p-4 md:p-8">
      <h1 className="text-3xl font-bold mb-4">Settings</h1>
      
      <div className="bg-white/5 shadow rounded-lg p-6 border border-gray-700">
        <div className="mb-4">
          <label className="block text-sm font-bold mb-2">Server URL</label>
          <input 
            type="text"
            className="w-full bg-white/10 border border-gray-600 rounded py-2 px-3 focus:outline-none focus:border-netover_blue text-netover_text"
            value={settings.serverUrl}
            onChange={(e) => setSettingsState({ ...settings, serverUrl: e.target.value })}
            placeholder="ws://localhost:3001"
          />
        </div>
        
        <div className="mt-6 flex items-center justify-between">
          <CustomButton text="Save Settings" onClick={handleSave} additionClass="bg-netover_blue text-white" />
          {status && <span className="text-netover_green font-bold text-sm">{status}</span>}
        </div>
      </div>
    </div>
  );
}
