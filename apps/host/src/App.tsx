import './App.css'
import { Routes, Route, useLocation, HashRouter } from 'react-router-dom'
import { AnimatePresence, motion } from "framer-motion";

import Home from './pages/Home';
import Pairing from './pages/Pairing';
import Manage from './pages/Manage';
import Launch from './pages/Launch';
import Config from './pages/Config';

const PageMotion = ({ children }: { children: React.ReactNode }) => {
  const location = useLocation();
  return (
    <motion.div
      key={location.pathname}
      className="p-10 bg-netover_bg text-netover_text min-h-screen"
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.25 }}
    >
      {children}
    </motion.div>
  );
};

function AppRoutes() {
  const location = useLocation();

  return (
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
          path="/pairing"
          element={
            <PageMotion>
              <Pairing />
            </PageMotion>
          }
        />
        <Route 
          path="/managekey"
          element={
            <PageMotion>
              <Manage />
            </PageMotion>
          }
        />
        <Route 
          path="/launch"
          element={
            <PageMotion>
              <Launch />
            </PageMotion>
          }
        />
        <Route 
          path="/config"
          element={
            <PageMotion>
              <Config />
            </PageMotion>
          }
        />
      </Routes>
    </AnimatePresence>
  );
}

export default function App() {
  return (
    <HashRouter>
      <AppRoutes />
    </HashRouter>
  );
}