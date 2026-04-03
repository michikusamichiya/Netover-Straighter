import { useState } from "react";
import { createUserWithEmailAndPassword } from "firebase/auth";
import { auth } from "@/firebase";

export default function Register() {
  const [email, setEmail] = useState("");
  const [pw, setPw] = useState("");
  const [confirmPw, setConfirmPw] = useState("");
  const [err, setErr] = useState("");
  const [success, setSuccess] = useState("");

  const handleRegister = async () => {
    setErr("");
    setSuccess("");

    if (pw !== confirmPw) {
      setErr("Passwords do not match!");
      return;
    }

    try {
      await createUserWithEmailAndPassword(auth, email, pw);
      setSuccess("Account created successfully!");
      setEmail("");
      setPw("");
      setConfirmPw("");
    } catch (error) {
      console.error("Registration failed:", error);
      setErr(error.message);
    }
  };

  return (
    <div className="p-10 max-w-md mx-auto">
      <h1 className="text-3xl font-bold mb-6">Register</h1>
      {err && <span className="text-red-500 block mb-2">{err}</span>}
      {success && <span className="text-green-500 block mb-2">{success}</span>}

      <div className="space-y-4">
        <input
          type="email"
          placeholder="Email"
          className="border p-2 w-full"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <input
          type="password"
          placeholder="Password"
          className="border p-2 w-full"
          value={pw}
          onChange={(e) => setPw(e.target.value)}
        />
        <input
          type="password"
          placeholder="Confirm Password"
          className="border p-2 w-full"
          value={confirmPw}
          onChange={(e) => setConfirmPw(e.target.value)}
        />

        <button
          type="button"
          onClick={handleRegister}
          className="px-4 py-2 bg-black text-white rounded w-full"
        >
          Register
        </button>
      </div>
    </div>
  );
}
