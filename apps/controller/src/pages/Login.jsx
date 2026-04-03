import { useState, useEffect } from "react";
import { signInWithEmailAndPassword, signOut, onAuthStateChanged } from "firebase/auth";
import { auth } from "@/firebase";
import { Link } from "react-router-dom";

export default function Login() {
  const [email, setEmail] = useState("");
  const [pw, setPw] = useState("");
  const [user, setUser] = useState(null);
  const [err, setErr] = useState("");

  // ログイン状態を監視
  useEffect(() => {
    const unsub = onAuthStateChanged(auth, (currentUser) => {
      setUser(currentUser);
      if (currentUser) {
        // ログイン済みならダッシュボードへ
        // navigate("/dashboard");
      }
    });
  }, []);

  const handleLogin = async () => {
    setErr("");
    try {
      await signInWithEmailAndPassword(auth, email, pw);
    } catch (error) {
      console.error("ログイン失敗:", error);
      let message = error.message;
      // Firebaseのauth/invalid-credentialエラーの場合は、分かりやすいメッセージを設定
      if (error.code === "auth/invalid-credential") {
        message = "The email address or password is incorrect.";
      }
      setErr(message);
    }
  };

  const handleLogout = async () => {
    await signOut(auth);
  };

  return (
    <div className="p-10 max-w-md mx-auto">
      <h1 className="text-3xl font-bold mb-6">Login</h1>
      {err && <span className="text-red-500">{err}</span>}
      {user ? (
        <div>
          <p>Logged in as: {user.email}</p>
          <button
            type="button"
            onClick={handleLogout}
            className="px-4 py-2 bg-netover_text text-netover_bg rounded"
          >
            Logout
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          <input
            type="email"
            placeholder="Email"
            className="border p-2 w-full bg-netover_bg text-netover_text"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
          <input
            type="password"
            placeholder="Password"
            className="border p-2 w-full bg-netover_bg text-netover_text"
            value={pw}
            onChange={(e) => setPw(e.target.value)}
          />

          <Link to="/register" className="block mt-6 text-blue-500 hover:underline">Haven't you created an account yet?</Link>
          <Link to="/reset/password" className="block text-blue-500 hover:underline">Forgot your password?</Link>
          <button
            type="button"
            onClick={handleLogin}
            className="px-4 py-2 bg-netover_text text-netover_bg rounded w-full"
          >
            Login
          </button>
        </div>
      )}
    </div>
  );
}
