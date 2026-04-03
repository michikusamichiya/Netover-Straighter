import React from "react";
import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import Warning from "../components/Warning";
import CustomButton from "../components/CustomButton";
import { Ban, Key, Pencil, Trash } from "lucide-react";
import { Modal } from "../components/Modal";
import { invoke } from "@tauri-apps/api/core";

export default function Manage() {
  const [keys, setKeys] = useState<Record<string, { available: boolean; body: string }>>({});
  const [open, setOpen] = useState(false);
  const [modalDom, setModalDom] = useState<React.ReactNode>(null);

  const load = async () => {
    try {
      const data = await invoke<Record<string, { available: boolean; body: string }>>("loadkeys", { isKeyIdOnly: false });
      if (data) setKeys(data);
    } catch (err) {
      console.error("Failed to load keys:", err);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const deleteModal = (id: string) => {
    setModalDom(
      <div className="flex flex-col gap-4 items-center px-2 py-1">
        <div>
          <span className="font-semibold px-1">Are you sure to delete this key:</span>
          <span className="ml-2 font-mono bg-gray-800 px-2 py-1 rounded">{id}</span>
        </div>
        <div className="flex gap-4 mt-2 justify-center">
          <button
            className="bg-red-600 hover:bg-red-700 text-white font-bold py-1 px-4 rounded transition"
            onClick={async (e) => {
              e.stopPropagation();
              try {
                await invoke("deletekey", { id });
                setOpen(false);
                setModalDom(null);
                load();
              } catch (err: any) {
                setModalDom(<div className="text-red-500">Failed to delete key: {err.toString()}</div>);
              }
            }}
          >
            Delete
          </button>
          <button
            className="bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold py-1 px-4 rounded transition"
            onClick={(e) => {
              e.stopPropagation();
              setOpen(false);
              setModalDom(null);
            }}
          >
            Cancel
          </button>
        </div>
      </div>
    );
    setOpen(true);
  };

  const editModal = (id: string) => {
    function EditKeyForm({ origId }: { origId: string }) {
      const [newId, setNewId] = useState(origId);
      const [error, setError] = useState("");
      const [loading, setLoading] = useState(false);

      const validate = (val: string) => /^[A-Z]{6}$/.test(val);

      const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError("");
        if (!validate(newId)) {
          setError("ID must be 6 uppercase A-Z letters.");
          return;
        }
        setLoading(true);
        try {
          await invoke("editkey", { id: origId, opt: { id: newId } });
          setOpen(false);
          setModalDom(null);
          load();
        } catch (err: any) {
          setError("Failed to change key ID: " + err.toString());
        } finally {
          setLoading(false);
        }
      };

      return (
        <form onSubmit={handleSubmit} className="flex flex-col gap-4 items-center px-2 py-1 w-64">
          <div className="text-center">
            <span className="font-semibold">Change Key ID</span>
            <div className="text-gray-400 mt-2 text-sm">
              Old ID: <span className="font-mono">{origId}</span>
            </div>
          </div>
          <div className="flex flex-col gap-2 w-full text-netover_bg">
            <label htmlFor="newId" className="text-left text-sm mb-1 text-netover_text">
              New ID<span className="text-red-500">*</span>
            </label>
            <input
              id="newId"
              type="text"
              value={newId}
              onChange={(e) => {
                setNewId(e.target.value.toUpperCase());
                setError("");
              }}
              className="border px-2 py-1 rounded font-mono text-lg tracking-widest uppercase outline-none focus:ring-2 focus:ring-netover_blue"
              maxLength={6}
              pattern="[A-Z]{6}"
              required
              disabled={loading}
              autoFocus
            />
          </div>
          {error && <div className="text-red-500 text-xs text-center">{error}</div>}
          <div className="flex gap-4 mt-2 justify-center w-full">
            <button
              type="submit"
              disabled={loading}
              className={`bg-netover_blue hover:bg-blue-600 text-white font-bold py-1 px-4 rounded transition ${
                loading ? "opacity-70 cursor-not-allowed" : ""
              }`}
            >
              {loading ? "Updating..." : "Update"}
            </button>
            <button
              type="button"
              className="bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold py-1 px-4 rounded transition"
              disabled={loading}
              onClick={(e) => {
                e.stopPropagation();
                setOpen(false);
                setModalDom(null);
              }}
            >
              Cancel
            </button>
          </div>
        </form>
      );
    }
    setModalDom(<EditKeyForm origId={id} />);
    setOpen(true);
  };

  const ban = async (id: string) => {
    try {
      await invoke("editkey", { id, opt: { avail: false } });
      await load();
    } catch (err: any) {
      alert("Failed to disable key: " + err.toString());
    }
  };

  const unban = async (id: string) => {
    try {
      await invoke("editkey", { id, opt: { avail: true } });
      await load();
    } catch (err: any) {
      alert("Failed to enable key: " + err.toString());
    }
  };

  return (
    <div className="p-4">
      <h1 className="text-3xl font-bold text-netover_blue py-3">Key Management</h1>
      <Modal dom={modalDom} flag={open} setFlag={setOpen}></Modal>
      <Link to="/" className="text-blue-500 underline mb-4 inline-block hover:text-blue-400">
        Go to Home
      </Link>
      <Warning
        title="Security Warning"
        message="These keys provide full access to your machine. Leaking them is equivalent to giving away your password. Never share or publish these keys."
      />

      {Object.keys(keys).length === 0 ? (
        <div className="py-12 text-gray-500 italic text-center text-xl">No paired keys found.</div>
      ) : (
        <div className="overflow-x-auto my-6 border border-gray-700 rounded-lg shadow-xl">
          <table className="table-auto w-full">
            <thead>
              <tr className="bg-gray-800 text-netover_text border-b border-gray-700">
                <th className="px-6 py-4 text-left font-semibold">Key ID</th>
                <th className="px-6 py-4 text-left font-semibold">Key (Base64)</th>
                <th className="px-6 py-4 text-center font-semibold">Actions</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(keys).map(([id, key]) => (
                <tr
                  key={id}
                  className={`border-b border-gray-800 transition-colors hover:bg-gray-900/50 ${
                    key.available ? "text-netover_text" : "text-gray-600 bg-gray-900/20"
                  }`}
                >
                  <td className="px-6 py-4 font-mono font-bold tracking-wider">{id}</td>
                  <td className="px-6 py-4 font-mono text-sm opacity-80 break-all">
                    {key.body.slice(0, 12)}
                    <span className="select-none opacity-30">{"*".repeat(Math.max(0, key.body.length - 12))}</span>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center justify-center gap-4">
                      <Pencil
                        className="w-5 h-5 cursor-pointer hover:text-netover_blue transition-transform active:scale-95"
                        onClick={() => editModal(id)}
                      />
                      {key.available ? (
                        <Ban
                          className="w-5 h-5 cursor-pointer hover:text-red-500 transition-transform active:scale-95"
                          onClick={() => ban(id)}
                        />
                      ) : (
                        <Key
                          className="w-5 h-5 cursor-pointer hover:text-netover_green transition-transform active:scale-95"
                          onClick={() => unban(id)}
                        />
                      )}
                      <Trash
                        className="w-5 h-5 cursor-pointer hover:text-red-600 transition-transform active:scale-95"
                        onClick={() => deleteModal(id)}
                      />
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="mt-8">
        <CustomButton text="Add New Machine (Pairing)" onClick={() => (window.location.href = "/pairing")} />
      </div>
    </div>
  );
}