import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { signOut } from "firebase/auth";
import { auth } from "@/firebase";

export default function Logout() {
  const navigate = useNavigate();

  useEffect(() => {
    const doLogout = async () => {
      try {
        await signOut(auth);
      } catch (err) {
        // ログアウト失敗しても強制的にログイン画面に遷移
        // 必要なら: console.error("Logout failed:", err);
      }
      navigate("/login", { replace: true });
    };
    doLogout();
  }, [navigate]);

  return (
    <div className="w-full h-full flex items-center justify-center">
      <div className="text-center">
        <div className="text-2xl font-semibold mb-4">Logging out...</div>
        <div className="text-neutral-500">Please wait.</div>
      </div>
    </div>
  );
}
