import { useState, useRef, useEffect } from "react";
import { Link } from "react-router-dom";
import { createLaunchService, STATUS, STATUS_TEXT } from "../../scripts/launchService";
import { Loader, X, Cog, Terminal, Wallpaper } from "lucide-react";
import ManageScreen from "./ManageScreen";

export default function Launch() {
  const [file, setFile] = useState(null);
  const [err, setErr] = useState("");
  const [status, setStatus] = useState(STATUS.INTERACTIVE);
  const [isActive, setIsActive] = useState(false);

  const [isOnFocus, setIsOnFocus] = useState(false);
  const [isOnLock, setIsOnLock] = useState(false);

  const fileInputRef = useRef(null);
  const launchServiceRef = useRef(null);

  useEffect(() => {
    launchServiceRef.current = createLaunchService({
      setErr: setErr,
      setStatus: setStatus,
      onActive: () => {
        setIsActive(true);
      },
      onUnactive: () => {
        setIsActive(false);
      },
    });
  }, []);

  const handleSubmit = async (e) => {
    setErr("");
    if (!file) {
      setErr("Please select a file to upload.");
      return;
    }
    try {
      if (!await launchServiceRef.current?.setup(file)) {
        return;
      }
      await launchServiceRef.current?.start();
    } catch (e) {
      console.log("An error has been occured");
      setErr(e.toString());
    }
  };

  const handleFileChange = (e) => {
    setFile(e.target.files[0] || null);
  };

  const handleFileClick = () => {
    fileInputRef.current?.click();
  };

  if (isActive) {
    return (
      <ManageScreen
        launchService={launchServiceRef.current}
        setIsOnFocus={setIsOnFocus}
        setIsOnLock={setIsOnLock}
        isOnLock={isOnLock}
      />
    )
  } else {
    return (
      <div className="p-10">
        <h1 className="text-3xl font-bold mb-6">Launch</h1>
        {
          (() => {
            if (status == STATUS.INTERACTIVE) {
              return (
                <form className="space-y-4" onSubmit={handleSubmit}>
                  <span className="text-red-500">{err}</span>
                  <div>
                    <input
                      ref={fileInputRef}
                      type="file"
                      className="hidden"
                      onChange={handleFileChange}
                      accept=".nok"
                    />
                    <button
                      type="button"
                      className={`
          flex items-center justify-center w-full px-4 py-3 rounded
          border-2 border-dashed border-gray-400 bg-netover_bg text-netover_text
          hover:border-netover_blue hover:bg-gray-50 transition
          focus:outline-none
        `}
                      onClick={handleFileClick}
                    >
                      {file ? (
                        <span className="truncate w-full">
                          <span className="font-bold text-netover_green">Selected:</span> {file.name}
                        </span>
                      ) : (
                        <span className="text-gray-500">Click to select a file to upload</span>
                      )}
                    </button>
                  </div>
                  <Link to="" className="block mt-6 text-blue-500 hover:underline">
                    Need help?
                  </Link>
                  <button
                    type="submit"
                    className="px-4 py-2 bg-netover_text text-netover_bg rounded w-full"
                    onClick={(e) => { e.preventDefault(); handleSubmit(); }}
                  >
                    Start Launch
                  </button>
                </form>
              );
            }
            if (status == STATUS.CONNECTING || status == STATUS.REQUESTING) {
              return (
                <>
                  <Loader className="animate-spin" />
                  <p>{STATUS_TEXT[status]}</p>
                </>
              );
            }
          })()
        }
      </div>
    );
  }
}


