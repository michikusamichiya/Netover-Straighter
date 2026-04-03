import { useState } from "react";
import { auth } from "@/firebase";
import { updateProfile } from "firebase/auth";

export default function ProfileModal({ user, isOpen, onClose }) {
  const [displayName, setDisplayName] = useState(user?.displayName || "");
  const [photoURL, setPhotoURL] = useState(user?.photoURL || "");
  const [loading, setLoading] = useState(false);
  const [msg, setMsg] = useState("");

  if (!isOpen) return null;

  const handleSave = async () => {
    if (!user) return;
    setLoading(true);
    setMsg("");
    try {
      const token = await user.getIdToken();
      const res = await fetch(
        `${import.meta.env.VITE_FILTERKILLER_WORKER_API}/account/update`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({
            displayName,
            imageURL: photoURL,
          }),
        }
      );
      console.log(res);
      if (!res.ok) {
        const errorMsg = (await res.json()).error || `Failed to update profile`;
        throw new Error(errorMsg);
      }
      setMsg("Profile updated!");
    } catch (err) {
      console.error(err);
      setMsg("Failed to update profile.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-lg w-96 p-6 relative">
        <button
          className="absolute top-3 right-3 text-gray-500 hover:text-gray-800"
          onClick={onClose}
        >
          ✕
        </button>
        <h2 className="text-xl font-bold mb-4">Edit Profile</h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium">Display Name</label>
            <input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              className="mt-1 block w-full border rounded p-2"
            />
          </div>
          <div>
            <label className="block text-sm font-medium">Profile Photo URL</label>
            <input
              type="text"
              value={photoURL}
              onChange={(e) => setPhotoURL(e.target.value)}
              className="mt-1 block w-full border rounded p-2"
            />
          </div>
        </div>

        {msg && <p className="mt-2 text-sm">{msg}</p>}

        <div className="mt-6 flex justify-end gap-2">
          <button
            onClick={handleSave}
            disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            {loading ? "Saving..." : "Save"}
          </button>
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
