import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import CustomButton from "../components/CustomButton.tsx";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { LoaderCircle } from "lucide-react";

const STATUS_TEXT: Record<string, string> = {
  initializing: "Initializing...",
  connecting: "Connecting to server...",
  waiting: "Waiting for connection request...",
  recievedreq: "Recieved Request, Checking",
  starting_core: "Starting core server...",
  activate: "Active",
};

export default function Launch() {
  const navigate = useNavigate();

  const [status, setStatus] = useState("initializing");
  const [error, setError] = useState("");
  const [timeLaunch] = useState(() => Date.now());
  const [now, setNow] = useState(() => Date.now());
  const [isActive, setIsActive] = useState(false);

  const formatDuration = (a: number, b: number) => {
    // a, b are ms, and a <= b is guaranteed
    let diffMs = b - a;
    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    diffMs -= hours * 1000 * 60 * 60;
    const minutes = Math.floor(diffMs / (1000 * 60));
    diffMs -= minutes * 1000 * 60;
    const seconds = Math.floor(diffMs / 1000);
    const pad = (num: number) => String(num).padStart(2, "0");
    return `${pad(hours)}h:${pad(minutes)}m:${pad(seconds)}s`;
  };

  const elapsedText = useMemo(() => formatDuration(timeLaunch, now), [timeLaunch, now]);

  useEffect(() => {
    let unlistenPromises: Promise<UnlistenFn>[] = [];

    const setupListeners = () => {
      unlistenPromises.push(
        listen("launching_connected", () => {
          setStatus("connecting");
        })
      );
      unlistenPromises.push(
        listen("launching_init_success", () => {
          setStatus("waiting");
        })
      );
      unlistenPromises.push(
        listen("launching_requested", () => {
          setStatus("recievedreq");
        })
      );
      unlistenPromises.push(
        listen("launching_queryverify", () => {
          setStatus("starting_core");
        })
      );
      unlistenPromises.push(
        listen("launching_rtc_open", () => {
          setStatus("activate");
          setIsActive(true);
        })
      );
      unlistenPromises.push(
        listen("launching_rtc_close", () => {
          setStatus("waiting");
          setIsActive(false);
          setError("");
        })
      );
      unlistenPromises.push(
        listen<string>("launching_error", (e) => {
          setError(e.payload);
        })
      );
    };

    setupListeners();

    // React StrictMode 対策（Pairing と同じ要領で invoke を遅延させる）
    const timer = setInterval(() => {
      setNow(Date.now());
    }, 1000);

    const startTimeout = setTimeout(() => {
      invoke("launch").catch((e: any) => setError(e.toString()));
    }, 100);

    // 終了/ページ遷移
    const handleBeforeUnload = () => {
      invoke("end_launching").catch(() => {});
    };
    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      clearInterval(timer);
      clearTimeout(startTimeout);
      window.removeEventListener("beforeunload", handleBeforeUnload);
      unlistenPromises.forEach((p) => p.then((unlisten) => unlisten()));
      invoke("end_launching").catch(() => {});
    };
  }, []);

  const eliminate = async () => {
    try {
      await invoke("end_launching");
    } catch {
      // ignore
    }
    navigate("/");
  };

  return (
    <div>
      <h1 className="text-3xl font-bold text-netover_blue py-3">Launching</h1>

      {(() => {
        if (!isActive) {
          return (
            <div className="flex flex-col item-center gap-2">
              {error === "" ? (
                <>
                  <div className="flex items-center space-x-2">
                    <LoaderCircle className="animate-spin" />
                    <div className="flex flex-col">
                      <span>{STATUS_TEXT[status] ?? status}</span>
                    </div>
                  </div>

                  <div className="my-3 bg-gray-800 font-bold p-3 rounded block">
                    {elapsedText}
                  </div>

                  <div className="font-bold">
                    Status: <span className={isActive ? "text-red-500" : "text-blue-500"}>{isActive ? "Active" : "Non-Active"}</span>
                  </div>

                  <CustomButton
                    text="Eliminate"
                    onClick={() => eliminate()}
                    additionClass="bg-netover_blue"
                  />
                </>
              ) : (
                <>
                  <span className="text-red-500">{error}</span>
                  <CustomButton
                    text="Go back Top"
                    onClick={() => eliminate()}
                    additionClass="bg-netover_blue"
                  />
                </>
              )}
            </div>
          );
        }

        return (
          <>
            <div className={`${isActive ? "text-red-500" : "text-blue-500"} font-bold`}>
              Status: {isActive ? "Active" : "Non-Active"}
            </div>
            <br />
            <ul>
              <li>If you can see the screen, it is success! :)</li>
              <li>LeftCtrl + RightShift + Q: Close the connection</li>
            </ul>
          </>
        );
      })()}
    </div>
  );
}
