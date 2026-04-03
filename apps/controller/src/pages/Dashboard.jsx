import { useState, useEffect } from "react";
import { onAuthStateChanged, sendEmailVerification } from "firebase/auth";
import { auth } from "@/firebase";
import { Link, useNavigate } from "react-router-dom";
import { TriangleAlert } from "lucide-react";
import ProfileModal from "@/components/ProfileModal";

export default function Dashboard() {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const navigate = useNavigate();

  const [isModalOpen, setIsModalOpen] = useState(false);

  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, (currentUser) => {
      setUser(currentUser);
      setLoading(false);
      if (!currentUser) {
        // navigate("/login");
      }
    });
    return () => unsubscribe();
  }, [navigate]);

  const handleSendVerification = async () => {
    if (!user) return;
    setSending(true);
    try {
      await sendEmailVerification(user);
      setSent(true);
    } catch (err) {
      console.error("Failed to send email:", err);
    } finally {
      setSending(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-[60vh] flex items-center justify-center">
        <div className="text-lg text-neutral-500">Loading...</div>
      </div>
    );
  }

  if (!user) return null;

  return (
    <div className="py-10 max-w-xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Dashboard</h1>
        <div className="bg-netover_bg rounded shadow p-6 space-y-4">
        <div>
          <span className="font-semibold text-netover_text">User ID:</span>
          <span className="ml-2 text-netover_text break-all">{user.uid}</span>
        </div>
        <div>
          <span className="font-semibold text-netover_text">Email:</span>
          <span className="ml-2 text-netover_text">{user.email || "-"}</span>
        </div>
        <div>
          <span className="font-semibold text-netover_text">Email Verified:</span>
          <span className="ml-2">{user.emailVerified ? "Yes" : "No"}</span>
        </div>
        <div>
          <span className="font-semibold text-netover_text">Account Created:</span>
          <span className="ml-2">{user.metadata?.creationTime ? (new Date(user.metadata.creationTime)).toLocaleString() : "-"}</span>
        </div>
        <div>
          <span className="font-semibold text-netover_text">Last Sign-In:</span>
          <span className="ml-2">{user.metadata?.lastSignInTime ? (new Date(user.metadata.lastSignInTime)).toLocaleString() : "-"}</span>
        </div>
      </div>

      {/* メール認証通知 */}
      {!user.emailVerified && (
        <div className="bg-netover_text p-5 mt-2 rounded shadow flex items-center justify-between">
          <div className="flex items-center gap-2">
            <TriangleAlert className="w-4 h-4"/>
            <span>Email not verified.</span>
          </div>
          {sent ? (
            <span className="text-green-600 font-semibold">Verification email sent!</span>
          ) : (
            <button
              onClick={handleSendVerification}
              disabled={sending}
              className="text-blue-500 hover:underline"
            >
              {sending ? "Sending..." : "Send verification email"}
            </button>
          )}
        </div>
      )}
      <div className="flex m-auto">
        <div className="mt-6 mr-6 flex gap-4">
          <button
            onClick={() => setIsModalOpen(true)}
            className="px-4 py-2 bg-green-500 text-white rounded"
          >
            Edit Profile
          </button>
        </div>
        <div className="mt-6 mr-6 flex gap-4">
          <Link to="/logout" className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600">
            Logout
          </Link>
        </div>
      </div>
      <ProfileModal
        user={user}
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
      />
    </div>
  );
}
