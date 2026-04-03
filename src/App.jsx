import { Routes, Route, useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";

import Header from "@/components/Header";
import Footer from "@/components/Footer";
import Home from "@/pages/Home";
import NotFound from "./pages/errors/NotFound";
import Forbidden from "./pages/errors/Forbidden";
import ServerError from "./pages/errors/ServerError";
import Login from "./pages/Login";
import Register from "./pages/Signup";
import Logout from "./pages/Logout";
import Dashboard from "./pages/Dashboard";
import Pairing from "./pages/Pairing";
import Launch from "./pages/launch/Launch";

// ページ切り替えアニメーション用ラッパー
const PageMotion = ({ children }) => (
  <motion.div
    key={children?.type?.name} // ページコンポーネント名で key 指定
    initial={{ opacity: 0, y: 10 }}
    animate={{ opacity: 1, y: 0 }}
    exit={{ opacity: 0, y: -10 }}
    transition={{ duration: 0.25 }}
  >
    {children}
  </motion.div>
);

export default function App() {
  const location = useLocation();

  return (
    <div className="w-full h-full flex flex-col min-h-screen bg-netover_bg text-netover_text">
      {/* ヘッダー固定 */}
      <Header />

      {/* ページコンテンツ */}
      <main className="flex-1 pt-14">
        <AnimatePresence mode="wait">
          <Routes location={location} key={location.pathname}>
            <Route
              path="/"
              element={
                <PageMotion>
                  <Home />
                </PageMotion>
              }
            />
            <Route
              path="/login"
              element={
                <PageMotion>
                  <Login />
                </PageMotion>
              }
            />
            <Route
              path="/register"
              element={
                <PageMotion>
                  <Register />
                </PageMotion>
              }
            />
            <Route
              path="/logout"
              element={
                <PageMotion>
                  <Logout />
                </PageMotion>
              }
            />
            <Route
              path="/dashboard"
              element={
                <PageMotion>
                  <Dashboard />
                </PageMotion>
              }
            />
            <Route
              path="/pairing"
              element={
                <PageMotion>
                  <Pairing />
                </PageMotion>
              }
            />
            <Route
              path="/access"
              element={
                <PageMotion>
                  <Launch />
                </PageMotion>
              }
            />
            {/* 404 */}
            <Route
              path="*"
              element={
                <PageMotion>
                  <NotFound />
                </PageMotion>
              }
            />
            {/* 403 */}
            <Route
              path="/403"
              element={
                <PageMotion>
                  <Forbidden />
                </PageMotion>
              }
            />
            {/* 500 */}
            <Route
              path="/500"
              element={
                <PageMotion>
                  <ServerError />
                </PageMotion>
              }
            />
          </Routes>
        </AnimatePresence>
      </main>

      {/* フッター */}
      <Footer />
    </div>
  );
}
