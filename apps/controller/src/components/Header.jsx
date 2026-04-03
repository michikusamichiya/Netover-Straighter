import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { onAuthStateChanged, signOut } from "firebase/auth";
import { auth } from "@/firebase";

export default function Header() {
  const [user, setUser] = useState(null);
  const [openWhenSmart, setOpenWhenSmart] = useState(false);

  useEffect(() => {
    const unsub = onAuthStateChanged(auth, (currentUser) => {
      setUser(currentUser);
    });
    return () => unsub();
  }, []);

  const handleLogout = async () => {
    try {
      await signOut(auth);
      // ログアウト後にトップページに飛ばしたい場合は Link または navigate で
      // navigate("/");
    } catch (err) {
      console.error("ログアウト失敗:", err);
    }
  };

  return (
    <header className="fixed top-0 left-0 w-full h-14 bg-white/80 backdrop-blur shadow z-50 flex items-center px-4 md:px-6">
      <div className="flex items-center justify-between w-full text-neutral-700">
        {/* 左側 */}
        <div className="flex items-center gap-4 md:gap-6">
          <Link to="/" className="font-bold text-base md:text-lg">
            NetOver
          </Link>
          <Link to="https://filterkiller-site.pages.dev" className="hidden md:block hover:text-black font-bold text-md">FilterKiller</Link>
          <nav className="hidden md:flex items-center gap-4 text-sm text-neutral-700">
            <Link to="https://filterkiller-site.pages.dev/guideline" className="hover:text-black">Guideline</Link>
            <a href="https://instefty.onrender.com" className="hover:text-black">Instefty</a>
          </nav>
        </div>

        {/* PC表示: 右側ナビゲーション */}
        <nav className="hidden md:flex items-center gap-4">
          <div className="text-sm text-neutral-700 flex items-center gap-4">
            <Link to="/tools" className="hover:text-black">Tools</Link>
            <Link to="/pairing" className="hover:text-black">Pairing</Link>
            <Link to="/access" className="hover:text-black">Remote Control</Link>
            <Link to="/help" className="hover:text-black">Tutorial</Link>
            {user ? (
              <>
                <Link to="/dashboard" className="hover:text-black">Dashboard</Link>
                <button 
                  onClick={handleLogout} 
                  className="hover:text-red-500"
                >
                  Logout
                </button>
              </>
            ) : (
              <Link to="/login" className="hover:text-black">Login</Link>
            )}
          </div>
        </nav>

        {/* スマホ表示: ハンバーガーメニューボタン */}
        <button 
          className="md:hidden text-2xl text-neutral-700 hover:text-black focus:outline-none"
          onClick={() => setOpenWhenSmart(!openWhenSmart)}
          aria-label="メニューを開く"
        >
          ☰
        </button>
      </div>

      {/* スマホ表示: ドロップダウンメニュー */}
      {openWhenSmart && (
        <nav className="absolute top-14 left-0 w-full bg-white/95 backdrop-blur shadow-lg md:hidden z-40">
          <div className="flex flex-col p-4 gap-3">
            <Link 
              to="/tools" 
              className="text-neutral-900 hover:text-black py-2 border-b border-neutral-200"
              onClick={() => setOpenWhenSmart(false)}
            >
              Tools (Redirect to GitHub)
            </Link>
            <Link 
              to="/pairing" 
              className="text-neutral-900 hover:text-black py-2 border-b border-neutral-200"
              onClick={() => setOpenWhenSmart(false)}
            >
              Pairing
            </Link>
            <Link 
              to="/access" 
              className="text-neutral-900 hover:text-black py-2 border-b border-neutral-200"
              onClick={() => setOpenWhenSmart(false)}
            >
              Remote Control
            </Link>
            <Link 
              to="/help" 
              className="text-neutral-900 hover:text-black py-2 border-b border-neutral-200"
              onClick={() => setOpenWhenSmart(false)}
            >
              Tutorial
            </Link>
            {user ? (
              <>
                <Link 
                  to="/dashboard" 
                  className="text-neutral-900 hover:text-black py-2 border-b border-neutral-200"
                  onClick={() => setOpenWhenSmart(false)}
                >
                  Dashboard
                </Link>
                <button 
                  onClick={() => {
                    handleLogout();
                    setOpenWhenSmart(false);
                  }} 
                  className="text-neutral-900 hover:text-red-500 py-2 text-left"
                >
                  Logout
                </button>
              </>
            ) : (
              <Link 
                to="/login" 
                className="text-neutral-900 hover:text-black py-2 border-b border-neutral-200"
                onClick={() => setOpenWhenSmart(false)}
              >
                Login
              </Link>
            )}
          </div>
        </nav>
      )}
    </header>
  );
}
