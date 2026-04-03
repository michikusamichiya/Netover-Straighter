import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CircleOff, Ellipsis, Loader, CircleCheckBig } from "lucide-react";
import CustomButton from "../components/CustomButton.tsx";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

const STATUS_TEXT: Record<string, string> = {
  generating: "Generating authentication keys...",
  connecting: "Connecting to server...",
  exchanging: "Exchanging keys...",
  waiting: "Waiting request from the machine to match...",
  waitingcheck: "Waiting the check...",
  denied: "You or the other machine denied the pairing.",
  disconnected: "You or the other machine disconnected the pairing.",
  allowedkey: "Generating Key...,",
  complete: "Completed!",
};

export default function Launch() {
  const navigate = useNavigate();

  const [status, setStatus] = useState("generating");
  const [randId, setRandId] = useState("");
  const [checkhash, setCheckhash] = useState("");
  const [err, setErr] = useState("");
  const [accept, setAccept] = useState(-1);
  const [accepted, setAccepted] = useState(-1); // -1 = waiting, 0 = denied, 1 = accepted

  useEffect(() => {
    let unlistenPromises: Promise<UnlistenFn>[] = [];

    const setupListeners = () => {
      unlistenPromises.push(
        listen("pairing_connected", () => {
          setStatus("connecting");
        })
      );
      unlistenPromises.push(
        listen<string>("pairing_rand_id", (e) => {
          setRandId(e.payload);
          setStatus("waiting");
        })
      );
      unlistenPromises.push(
        listen<string>("pairing_exchange_from_local", (e) => {
          setCheckhash(e.payload);
          setStatus("waitingcheck");
        })
      );
      unlistenPromises.push(
        listen("pairing_allowed_key", () => {
          setStatus("allowedkey");
        })
      );
      unlistenPromises.push(
        listen("pairing_complete", () => {
          setStatus("complete");
        })
      );
      unlistenPromises.push(
        listen("pairing_accept_from_local", () => {
          setAccepted(1);
        })
      );
      unlistenPromises.push(
        listen("pairing_deny_from_local", () => {
          setAccepted(0);
          setStatus("denied");
          setErr("The other machine denied the pairing (other user handled)");
        })
      );
      unlistenPromises.push(
        listen("pairing_cancel_from_local", () => {
          setAccepted(-1);
        })
      );
      unlistenPromises.push(
        listen("pairing_disconnected_from_local", () => {
          setStatus("disconnected");
        })
      );
      unlistenPromises.push(
        listen<string>("pairing_error", (e) => {
          setErr(e.payload);
        })
      );
      unlistenPromises.push(
        listen<string>("pairing_disconnected", () => {
          setStatus("disconnected");
        })
      );

      // Use a timeout to debounce React StrictMode double mounts.
      // The first mount will be unmounted before the timeout completes,
      // destroying the promise and avoiding a race condition in the Rust backend.
      const timer = setTimeout(() => {
        invoke("start_pairing").catch((e: any) => {
          setErr(e.toString());
        });
      }, 100);

      return timer;
    };

    const timerObj = setupListeners();

    return () => {
      clearTimeout(timerObj);
      unlistenPromises.forEach((p) => p.then((unlisten) => unlisten()));
      invoke("end_pairing").catch(console.error);
    };
  }, []);

  useEffect(() => {
    if (status === "allowedkey" && randId) {
      invoke("save_key", { remoteId: randId }).catch((e) => setErr(e.toString()));
    }
  }, [status, randId]);

  const handleAccept = async () => {
    setAccept(1);
    try {
      await invoke("accept");
    } catch (e: any) {
      setErr(e.toString());
    }
  };

  const handleDeny = async () => {
    setAccept(0);
    setStatus("denied");
    setErr("You denied the pairing request (user handled)");
    try {
      await invoke("deny");
    } catch (e: any) {
      setErr(e.toString());
    }
  };

  const handleCancel = async () => {
    setAccept(-1);
    try {
      await invoke("cancel");
    } catch (e: any) {
      setErr(e.toString());
    }
  };

  const acceptStatus = (stat: number) => {
    if (stat === -1)
      return (
        <>
          <Ellipsis className="w-4 inline-block" />
          Waiting
        </>
      );
    if (stat === 0)
      return (
        <>
          <CircleOff className="w-4 inline-block" />
          Denied
        </>
      );
    if (stat === 1)
      return (
        <>
          <CircleCheckBig className="w-4 inline-block" />
          Accepted
        </>
      );
  };

  return (
    <div>
      <h1 className="text-3xl font-bold text-netover_blue py-3">Pairing</h1>

      <div className="flex flex-col items-center gap-2">
        {(() => {
          if (status === "waiting") {
            return (
              <>
                <span className="text-xl">
                  Enter this ID in the computer to match
                </span>
                <div className="bg-gray-800 font-bold p-3 text-4xl rounded">
                  {randId}
                </div>
              </>
            );
          }
          if (status === "waitingcheck")
            return (
              <>
                Check both of computers view the same text:
                <div className="bg-gray-800 font-bold p-3 text-xl rounded">
                  {checkhash}
                </div>
                <div>
                  {accept === -1 ? (
                    <>
                      <button
                        className="w-full px-4 py-2 bg-netover_green text-netover_text rounded mt-2"
                        onClick={handleAccept}
                      >
                        Accept
                      </button>
                      <button
                        className="w-full px-4 py-2 bg-red-500 text-netover_text rounded mt-2"
                        onClick={handleDeny}
                      >
                        Deny
                      </button>
                    </>
                  ) : (
                    <>
                      <button
                        className="px-4 py-2 bg-netover_text text-netover_bg rounded mt-2 w-full"
                        onClick={handleCancel}
                      >
                        Cancel Accepting
                      </button>
                    </>
                  )}
                </div>
                <div>Controller: {acceptStatus(accepted)}</div>
              </>
            );
          if (status === "denied") {
            return (
              <>
                <p>The other machine denied the pairing.</p>
              </>
            );
          }
          if (status === "complete") {
            return (
              <>
                <div className="text-4xl font-bold bg-gradient-to-r from-red-500 via-yellow-500 via-green-500 via-blue-500 to-purple-500 bg-clip-text text-transparent select-none">
                  COMPLETED!
                </div>
              </>
            );
          }
          return null;
        })()}

        {status !== "complete" && (
          <>
            <Loader className="animate-spin" />
            <p>{STATUS_TEXT[status]}</p>
          </>
        )}

        {err && (
          <>
            <p className="text-red-500">{err}</p>
          </>
        )}

        <CustomButton
          text="Go back Top"
          onClick={() => {
            invoke("end_pairing").catch(console.error);
            navigate("/");
          }}
          additionClass="bg-netover_blue"
        />
      </div>
    </div>
  );
}