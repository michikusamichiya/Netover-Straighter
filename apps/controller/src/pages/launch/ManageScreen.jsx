import { useState, useEffect, useRef } from "react";
import { getSettings } from "@/scripts/settings";
import { Loader, X, Cog, Terminal, Wallpaper } from "lucide-react";

export default function ManageScreen({ launchService, setIsOnFocus, setIsOnLock, isOnLock }) {
    const canvasRef = useRef(null);
    const videoHiddenRef = useRef(null);
    const audioRef = useRef(null);
    const requestRef = useRef(null);

    const lastMouseMoveRef = useRef(0);
    const isOnLockRef = useRef(isOnLock);

    const mouseFPS = 30;

    const [gameMode, setGameMode] = useState(false);
    const [isGameModePaused, setIsGameModePaused] = useState(false);
    const isGameModePausedRef = useRef(false);
    
    const [settings, setSettings] = useState({});
    const settingsRef = useRef({});

    useEffect(() => {
        const updateSettings = () => {
            const newSettings = getSettings();
            setSettings(newSettings);
            settingsRef.current = newSettings;
            setGameMode(newSettings.gameMode);
        };
        updateSettings(); // Initial load
        window.addEventListener("storage", updateSettings);
        return () => window.removeEventListener("storage", updateSettings);
    }, []);

    useEffect(() => {
        isOnLockRef.current = isOnLock;
        if (isOnLock && document.pointerLockElement) {
            document.exitPointerLock();
        }
    }, [isOnLock]);

    useEffect(() => {
        isGameModePausedRef.current = isGameModePaused;
    }, [isGameModePaused]);

    useEffect(() => {
        if (!launchService || !videoHiddenRef.current) return;
        const stream = launchService.getStream();
        const video = videoHiddenRef.current;
        video.srcObject = stream;

        // トラックが追加されたら再生を試みる
        stream.addEventListener("addtrack", () => {
            video.srcObject = stream;
            video.play().catch(e => console.log("play failed:", e));
        });

        video.play().catch(e => console.log("Video autoPlay prevented:", e));
    }, [launchService]);
    useEffect(() => {
        if (!launchService || !audioRef.current) return;
        const stream = launchService.getStream();
        const audio = audioRef.current;
        audio.srcObject = stream;

        stream.addEventListener("addtrack", () => {
            audio.srcObject = stream;
            audio.play().catch(e => console.log("play failed:", e));
        });
    }, [launchService]);

    useEffect(() => {
        const video = videoHiddenRef.current;
        const canvas = canvasRef.current;
        if (!video || !canvas) return;
        const ctx = canvas.getContext("2d");

        const renderLoop = () => {
            if (video.readyState >= video.HAVE_CURRENT_DATA) {
                if (video.videoWidth > 0 && video.videoHeight > 0) {
                    if (canvas.width !== video.videoWidth || canvas.height !== video.videoHeight) {
                        canvas.width = video.videoWidth;
                        canvas.height = video.videoHeight;
                    }
                    try {
                        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
                    } catch (e) {
                        // Ignore drawImage errors
                    }
                }
            }
            requestRef.current = requestAnimationFrame(renderLoop);
        };

        requestRef.current = requestAnimationFrame(renderLoop);

        return () => {
            if (requestRef.current) {
                cancelAnimationFrame(requestRef.current);
            }
        };
    }, []);

    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.code === "ControlRight") {
                console.log("Locked");
                setIsOnLock(!isOnLockRef.current);
                return;
            }
            
            const leaveCode = settingsRef.current?.key?.gamemode?.leave?.code || "F2";
            const enterCode = settingsRef.current?.key?.gamemode?.enter?.code || "F8";

            if (e.code === leaveCode && gameMode) {
                setIsGameModePaused(true);
                if (document.pointerLockElement) {
                    document.exitPointerLock();
                }
            } else if (e.code === enterCode && gameMode) {
                setIsGameModePaused(false);
                if (!isOnLockRef.current && canvasRef.current) {
                    canvasRef.current.requestPointerLock();
                }
            }

            if (isOnLock) return;
            e.preventDefault();
            // if (e.repeat) return;

            // Toggle lock logic - allow RightCtrl regardless of lock state

            if (isOnLockRef.current) return;

            console.log(e.code);

            
            launchService.sendData(`KEY_DOWN ${e.code}`);
        };

        const handleKeyUp = (e) => {
            if (isOnLock) return;
            e.preventDefault();
            // console.log(e.code);
            if (isOnLockRef.current) return;
            launchService.sendData(`KEY_UP ${e.code}`);
        };

        const handleMouseUp = (e) => {
            if (isOnLock) return;
            if (e.target !== canvasRef.current && document.pointerLockElement !== canvasRef.current) return;
            e.preventDefault();
            // console.log(e.code);
            if (isOnLockRef.current) return;
            launchService.sendData(`MOUSE_CLICK_UP ${e.button}`);
        };

        const handleMouseDown = (e) => {
            if (isOnLock) return;
            if (e.target !== canvasRef.current && document.pointerLockElement !== canvasRef.current) return;
            e.preventDefault();
            // console.log(e.code);
            if (isOnLockRef.current) return;
            launchService.sendData(`MOUSE_CLICK_DOWN ${e.button}`);
        };
        const handleWheel = (e) => {
            if (isOnLock) return;
            if (e.target !== canvasRef.current && document.pointerLockElement !== canvasRef.current) return;
            e.preventDefault();
            
            const sensX = settingsRef.current.mouse?.wheel?.x ?? 1;
            const sensY = settingsRef.current.mouse?.wheel?.y ?? 1;
            launchService.sendData(`MOUSE_WHEEL ${e.deltaX * sensX} ${e.deltaY * sensY}`);
        }

        window.addEventListener("keydown", handleKeyDown);
        window.addEventListener("keyup", handleKeyUp);
        window.addEventListener("mousedown", handleMouseDown);
        window.addEventListener("mouseup", handleMouseUp);
        window.addEventListener("wheel", handleWheel);

        return () => {
            window.removeEventListener("keydown", handleKeyDown);
            window.removeEventListener("keyup", handleKeyUp);
            window.removeEventListener("mousedown", handleMouseDown);
            window.removeEventListener("mouseup", handleMouseUp);
            window.removeEventListener("wheel", handleWheel);
        };
    }, [launchService, setIsOnLock]);

    const handleMouseMove = (e) => {
        if (isOnLock) return;

        const now = Date.now();
        if (now - lastMouseMoveRef.current < 1000 / mouseFPS) return;
        lastMouseMoveRef.current = now;

        if (canvasRef.current && launchService) {
            const rect = canvasRef.current.getBoundingClientRect();
            let x = (e.clientX - rect.left) / rect.width;
            let y = (e.clientY - rect.top) / rect.height;

            // Normalize to [0,1]
            x = Math.max(0, Math.min(1, x));
            y = Math.max(0, Math.min(1, y));

            if (!gameMode || isGameModePausedRef.current) {
                launchService.sendData(`MOUSE_MOVE ${x} ${y}`);
            } else {
                const sensX = settingsRef.current.mouse?.sensitivity?.x ?? 1;
                const sensY = settingsRef.current.mouse?.sensitivity?.y ?? 1;
                launchService.sendData(`MOUSE_MOVE_RELATIVE ${e.movementX * sensX} ${e.movementY * sensY}`);
            }
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex flex-col bg-black">
            <header className="flex justify-left items-center p-2 bg-gray-900 border-b border-gray-800">
                <button className="px-4 py-2 bg-netover_text text-netover_bg mr-2 rounded" onClick={() => {
                    launchService?.reset();
                }}>
                    <X className="w-4 h-4" />
                </button>
                <button className="px-4 py-2 bg-netover_text text-netover_bg mr-2 rounded" onClick={() => {
                    window.open('/config', 'configPopup', 'width=600,height=700');
                }}>
                    <Cog className="w-4 h-4" />
                </button>
                <button className="px-4 py-2 bg-netover_text text-netover_bg mr-2 rounded" onClick={() => {

                }}>
                    <Terminal className="w-4 h-4" />
                </button>
                <button className="px-4 py-2 bg-netover_text text-netover_bg rounded" onClick={() => {
                }}>
                    <Wallpaper className="w-4 h-4" />
                </button>
            </header>
            <main className="flex-1 relative flex items-center justify-center overflow-hidden">
                <video
                    ref={videoHiddenRef}
                    autoPlay
                    playsInline
                    muted
                    style={{ position: "absolute", width: 1, height: 1, opacity: 0, pointerEvents: "none" }}
                />
                <audio
                    ref={audioRef}
                    autoPlay
                    style={{ position: "absolute", width: 1, height: 1, opacity: 0, pointerEvents: "none" }}
                />
                <canvas
                    ref={canvasRef}
                    className="max-w-full max-h-full object-contain"
                    onMouseMove={handleMouseMove}
                    onContextMenu={(e) => e.preventDefault()}
                    onClick={() => {
                        if (gameMode && !isOnLock && !isGameModePausedRef.current && canvasRef.current) {
                            canvasRef.current.requestPointerLock();
                        }
                    }}
                />

                {isOnLock && (
                    <div className="absolute top-4 left-4 pointer-events-none text-red-500 font-bold select-none">
                        <div className="text-2xl">REMOTE LOCKING</div>
                        <div className="text-lg">Press RightCtrl to unlock</div>
                    </div>
                )}
            </main>
        </div>
    )
}