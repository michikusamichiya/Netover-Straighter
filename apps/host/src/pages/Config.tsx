import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import CustomButton from "../components/CustomButton.tsx";

interface AppConfig {
  server_url: string;
}

export default function Config() {
  const navigate = useNavigate();
  const [target, setTarget] = useState<AppConfig>({ server_url: "" });
  const [status, setStatus] = useState<string>("");

  useEffect(() => {
    invoke<AppConfig>("get_config").then((cfg) => {
      setTarget(cfg);
    }).catch(err => {
      console.error(err);
      setStatus("Failed to load: " + err);
    });
  }, []);

  const saveConfig = async () => {
    try {
      await invoke("set_config", { config: target });
      setStatus("Saved successfully!");
      setTimeout(() => setStatus(""), 3000);
    } catch (err) {
      console.error(err);
      setStatus("Failed to save: " + err);
    }
  };

  return (
    <div>
      <h1 className="text-3xl font-bold py-3">Settings</h1>
      <p className="text-sm py-1 mb-4">Configure your NetOver Straighter settings here.</p>
      
      <div className="mb-4">
        <label className="block text-sm font-bold mb-2">Server URL</label>
        <input 
          type="text" 
          value={target.server_url} 
          onChange={(e) => setTarget({ ...target, server_url: e.target.value })} 
          placeholder="ws://localhost:3000"
          className="shadow appearance-none border rounded w-full py-2 px-3 text-black leading-tight focus:outline-none focus:shadow-outline text-netover_text"
        />
      </div>

      <div className="flex gap-2 mt-4">
        <CustomButton text="Save" onClick={saveConfig} additionClass="bg-netover_blue text-white" />
        <CustomButton text="Back" onClick={() => navigate("/")} additionClass="bg-gray-500 text-white" />
      </div>

      {status && <p className="mt-4 text-sm font-bold text-netover_green">{status}</p>}
    </div>
  );
}
