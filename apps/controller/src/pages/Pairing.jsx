import { useState, useEffect, useRef } from "react";
import { Link, useNavigate } from "react-router-dom";
import { CircleOff, Ellipsis, Loader, CircleCheckBig  } from "lucide-react";
import { createPairingService, STATUS, STATUS_TEXT } from "@/scripts/pairingService";
import CustomButton from "@/components/CustomButton";
import Warning from "@/components/Warning";

export default function Pairing() {
  const navigate = useNavigate();

  const [id, setID] = useState("");
  const [err, setErr] = useState("");

  const [status, setStatus] = useState(STATUS.INTERACTIVE);

  const [checkhash, setCheckhash] = useState(null);
  const [accepted, setAccepted] = useState(-1);
  const [accept, setAccept] = useState(false);
  
  const pairingServiceRef = useRef(null);

  const handleSubmit = async () => {
    setErr("");
    try {
      await pairingServiceRef.current?.start(id);
    } catch (error) {
      console.error("Pairing start error:", error);
      setErr(error.message || "Failed to start pairing");
    }
  };

  const acceptStatus = (stat) => {
    console.log(accepted);
    if (stat == -1) return (
      <>
        <Ellipsis className="w-4 inline-block" />
        <span className="mt-0">Waiting</span>
      </>
    );
    if (stat == 0) return (
      <>
        <CircleOff className="w-4 inline-block mt-0" />
        <span className="mt-0">Denied</span>
      </>
    );
    if (stat == 1) return (
      <>
        <CircleCheckBig className="w-4 inline-block" />
        <span className="mt-0">Accepted</span>
      </>
    );
  };

  useEffect(() => {
    pairingServiceRef.current = createPairingService({
      onStatusChange: setStatus,
      onError: setErr,
      setCheckhash: setCheckhash,
      setAccepted: setAccepted,
      setAccept: setAccept
    });

    return () => pairingServiceRef.current?.close();
  }, []);

  return (
    <div className="p-10">
      <h1 className="text-3xl font-bold mb-6">Pairing</h1>
      {
        (() => {
          if (status == STATUS.INTERACTIVE) {
            return (
            <div className="space-y-4">
              <span className="text-red-500">{err}</span>
              <input
                placeholder="ID"
                className="border p-2 w-full bg-netover_bg text-netover_text"
                value={id}
                onChange={(e) => setID(e.target.value)}
                maxLength={6}
              />
      
              <Link to="" className="block mt-6 text-blue-500 hover:underline">Need help?</Link>
              <button
                type="button"
                onClick={handleSubmit}
                className="px-4 py-2 bg-netover_text text-netover_bg rounded w-full"
              >
                Start matching
              </button>
            </div>
            )
          } if (status == STATUS.WAITING_FOR_CHECK) {
            return (
              <div className="space-y-4">
                <Loader className="animate-spin" />
                <p>{STATUS_TEXT[status]}</p>
                Check both of computers view the same text:
                <div className="bg-gray-800 font-bold p-3 text-xl rounded text-center">
                  {checkhash}
                </div>
                {
                  !accept && (
                    <>
                      <button className="px-4 py-2 bg-netover_green text-netover_text rounded w-full" onClick={() => { pairingServiceRef.current?.handleAccept(); }}>Accept</button>
                      <button className="px-4 py-2 bg-red-500 text-netover_text rounded w-full" onClick={() => { pairingServiceRef.current?.handleDeny(); }}>Deny</button>
                    </>
                  ) || (
                    <button className="px-4 py-2 bg-netover_text text-netover_bg rounded w-full" onClick={() => { pairingServiceRef.current?.handleCancel(); }}>Cancel Accepting</button>
                  )
                }
                <div>
                  Target: {acceptStatus(accepted)}
                </div>
              </div>
            )
          } else if (status == STATUS.CONNECTION_REFUSED) {
            return (
              <span>You or the target connection refused.</span>
            )
          } else if (status == STATUS.DENIED) {
            return (
              <span>You or the target denied the pairing.</span>
            )
          } else if (status == STATUS.COMPLETE) {
            return (
              <>
                <div className="text-4xl font-bold bg-gradient-to-r from-red-500 via-yellow-500 via-green-500 via-blue-500 to-purple-500 bg-clip-text text-transparent select-none">
                  COMPLETED!
                </div>
                <Warning
                  title={"Warning on Storage"}
                  message={
                    "Do not share this key with anyone, and do not edit the file. If this key is ever leaked, there is a risk that a third party could take unauthorized control of your Target."
                  }
                />
              </>
            )
          } else {
            return (
              <>
                <Loader className="animate-spin" />
                <p>{STATUS_TEXT[status]}</p>
              </>
            )
          }
        })()
      }
      {
        status != STATUS.INTERACTIVE && <>
        <CustomButton
          text="Go back Top"
          onClick={() => navigate("/")}
          additionClass="bg-netover_blue"
        />
      </>
      }
    </div>
  );
}
