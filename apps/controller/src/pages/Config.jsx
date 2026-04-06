import { useState, useEffect } from "react";
import { getSettings, saveSettings } from "@/scripts/settings";
import CustomButton from "@/components/CustomButton";

const CONFIG_FIELDS = [
  {
    id: "serverUrl",
    label: "Server URL",
    type: "text",
    placeholder: "ws://localhost:3001"
  },
  {
    label: "Mouse wheel settings",
    children: [
      {
        id: "mouse.wheel.x",
        label: "Mouse Sensitivity Wheel X",
        type: "number"
      },
      {
        id: "mouse.wheel.y",
        label: "Mouse Sensitivity Wheel Y",
        type: "number"
      }
    ]
  },
  {
    label: "Game Mode Settings",
    children: [
      {
        id: "gameMode",
        label: "Gamemode Enable (Pointer Lock)",
        type: "checkbox"
      },
      {
        id: "gamemode.key.leave.code",
        label: "Gamemode Leave Key (Code: e.g. F2)",
        type: "text",
        placeholder: "F2"
      },
      {
        id: "gamemode.key.enter.code",
        label: "Gamemode Enter Key (Code: e.g. F8)",
        type: "text",
        placeholder: "F8"
      },
      {
        label: "Settings in gameMode",
        children: [
          {
            id: "gamemode.mouse.sensitivity.x",
            label: "Mouse Sensitivity X",
            type: "number"
          },
          {
            id: "gamemode.mouse.sensitivity.y",
            label: "Mouse Sensitivity Y",
            type: "number"
          },
          {
            id: "gamemode.mouse.wheel.x",
            label: "Mouse Sensitivity Wheel X",
            type: "number"
          },
          {
            id: "gamemode.mouse.wheel.y",
            label: "Mouse Sensitivity Wheel Y",
            type: "number"
          },
          {
            id: "gamemode.key.leave.code",
            label: "Key to leave from game mode temporarily",
            type: "text"
          },
          {
            id: "gamemode.key.enter.code",
            label: "Key to enter game mode (when leaving temporarily)",
            type: "text"
          }
        ]
      }
    ]
  }
];

const getValue = (obj, path) => {
  if (!path) return undefined;
  return path.split('.').reduce((acc, part) => acc && acc[part], obj);
};

const setValue = (obj, path, value) => {
  if (!path) return obj;
  const parts = path.split('.');
  const newObj = { ...obj };
  let current = newObj;
  for (let i = 0; i < parts.length - 1; i++) {
    if (!current[parts[i]] || typeof current[parts[i]] !== 'object') {
      current[parts[i]] = {};
    } else {
      current[parts[i]] = { ...current[parts[i]] };
    }
    current = current[parts[i]];
  }
  current[parts[parts.length - 1]] = value;
  return newObj;
};

const ConfigField = ({ field, settings, handleChange, depth = 0 }) => {
  if (field.children) {
    return (
      <details className={`mb-4 w-full`} open>
        <summary className="font-bold cursor-pointer mb-2 text-netover_text hover:text-netover_blue transition bg-white/5 p-2 rounded select-none border border-gray-700">
          {field.label}
        </summary>
        <div className="pl-4 mt-2 border-l-2 border-gray-600/50">
          {field.children.map((child, idx) => (
            <ConfigField key={child.id || child.label || idx} field={child} settings={settings} handleChange={handleChange} depth={depth + 1} />
          ))}
        </div>
      </details>
    );
  }

  const value = getValue(settings, field.id);

  return (
    <div className={`mb-4 ${depth > 0 ? 'pr-2' : ''}`}>
      {field.type === "text" && (
        <>
          <label className="block text-sm font-bold mb-2">{field.label}</label>
          <input 
            type="text"
            className="w-full bg-white/10 border border-gray-600 rounded py-2 px-3 focus:outline-none focus:border-netover_blue text-netover_text"
            value={value !== undefined ? value : ""}
            onChange={(e) => handleChange(field.id, e.target.value)}
            placeholder={field.placeholder}
          />
        </>
      )}
      {field.type === "number" && (
        <>
          <label className="block text-sm font-bold mb-2">{field.label}</label>
          <input 
            type="number"
            step="0.1"
            className="w-full bg-white/10 border border-gray-600 rounded py-2 px-3 focus:outline-none focus:border-netover_blue text-netover_text"
            value={value !== undefined ? value : ""}
            onChange={(e) => handleChange(field.id, e.target.value !== "" ? parseFloat(e.target.value) : "")}
            placeholder={field.placeholder}
          />
        </>
      )}
      {field.type === "checkbox" && (
        <label className="flex items-center space-x-2 text-sm font-bold mb-2 cursor-pointer">
          <input 
            type="checkbox"
            className="form-checkbox h-4 w-4 text-netover_blue bg-white/10 border-gray-600 rounded focus:ring-netover_blue"
            checked={!!value}
            onChange={(e) => handleChange(field.id, e.target.checked)}
          />
          <span>{field.label}</span>
        </label>
      )}
    </div>
  );
};

export default function Config() {
  const [settings, setSettingsState] = useState({});
  const [status, setStatus] = useState("");

  useEffect(() => {
    setSettingsState(getSettings());
  }, []);

  const handleSave = () => {
    saveSettings(settings);
    setStatus("Settings saved successfully!");
    setTimeout(() => setStatus(""), 3000);
  };

  const handleChange = (id, value) => {
    setSettingsState(prev => setValue(prev, id, value));
  };

  return (
    <div className="max-w-2xl mx-auto p-4 md:p-8">
      <h1 className="text-3xl font-bold mb-4">Settings</h1>
      
      <div className="bg-white/5 shadow rounded-lg p-6 border border-gray-700">
        {CONFIG_FIELDS.map((field, idx) => (
           <ConfigField key={field.id || field.label || idx} field={field} settings={settings} handleChange={handleChange} />
        ))}
        
        <div className="mt-6 flex items-center justify-between">
          <CustomButton text="Save Settings" onClick={handleSave} additionClass="bg-netover_blue text-white" />
          {status && <span className="text-netover_green font-bold text-sm">{status}</span>}
        </div>
      </div>
    </div>
  );
}
