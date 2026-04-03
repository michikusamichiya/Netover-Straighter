import React from "react";
import { base64ToBytes, hmacSign } from "./crypto";
import { PercentDiamondIcon } from "lucide-react";

export const STATUS = {
  INTERACTIVE: "interactive",
  CONNECTING: "connecting",
  REQUESTING: "requesting"
};

export const STATUS_TEXT = {
  [STATUS.INTERACTIVE]: "",
  [STATUS.CONNECTING]: "Connecting to server...",
  [STATUS.REQUESTING]: "Requesting to access..."
};

export function createLaunchService({
  setErr,
  setStatus,
  onActive,
  onUnactive
}) {
  let wss = null;
  let nok = null;
  /**
   * @type {RTCPeerConnection}
   */
  let connection = null;
  /**
   * @type {{ [label: string]: RTCDataChannel }}
   */
  let channels = null;
  let pendingCandidates = [];
  let localIceq = [];
  let active = false;
  let stream = new MediaStream();

  const setup = async (file) => {
    if (!file) {
      setErr("No file");
      return false;
    }
    const text = await file.text();
    const lines = text.split('\n');
    try {
      const id = lines[0];
      const body = lines[1];
      const key = base64ToBytes(body);

      nok = {
        id: id,
        key: key
      };
      return true;
    } catch (e) {
      setErr("Invalid format");
      return false;
    }
  };
  const start = async () => {
    setStatus(STATUS.CONNECTING);
    wss = new WebSocket(`${import.meta.env.VITE_WEBSOCKET_SERVER}/ws/launch/controller`);

    wss.onopen = async () => {
      try {
        setStatus(STATUS.REQUESTING);
        wss?.send(JSON.stringify({
          type: "request",
          id: nok.id
        }));
      } catch (e) {
        setErr("Invalid format");
        await reset();
      }
    };

    wss.onmessage = async (msg) => {
      try {
        const rawData = msg.data;
        const data = JSON.parse(rawData);
        console.log(data);
        switch (data.type) {
          case "queryverify": {
            const nonce = Uint8Array.from(data.nonce);
            const hmac = await hmacSign(nok.key, nonce);
            const hmacToSend = Array.from(hmac);
            console.log("key: ", nok.key, "nonce: ", nonce, "hmac: ", hmacToSend);

            wss.send(JSON.stringify({
              type: "sendsign",
              sign: hmacToSend
            }));
            break;
          }
          case "deny": {
            setErr("Invalid key, make sure this is the correct key");
            await reset();
            break;
          }
          case "accept-offer": {
            connection = new RTCPeerConnection({
              iceServers: [
                { urls: "stun:stun.l.google.com:19302" },
                { urls: "stun:stun1.l.google.com:19302" },
                { urls: "stun:stun2.l.google.com:19302" },
                { urls: "stun:stun3.l.google.com:19302" },
                { urls: "stun:stun4.l.google.com:19302" },
                { urls: "stun:stun.stunprotocol.org:3478" },
                { urls: "stun:stun.ekiga.net" },
                { urls: "stun:stun.ideasip.com" },
                { urls: "stun:stun.schlund.de" },
                { urls: "stun:stun.voiparound.com" },
                { urls: "stun:stun.voipbuster.com" },
                { urls: "stun:stun.voipstunt.com" },
                { urls: "stun:stun.voxgratia.org" }
              ]
            });
            connection.ondatachannel = (channel) => {
              if (!channels) channels = {};
              channels[channel.channel.label] = channel.channel;
              channel.channel.onopen = () => {
                console.log("RTC Open (answer");
                active = true;
                onActive();
              };
              channel.channel.onclose = () => {
                active = false;
                onUnactive();
              };
              console.log(channels);
            };
            connection.onicecandidate = (ice) => {
              if (!ice.candidate) return;
              if (!connection) return;
            
              if (connection.localDescription) {
                wss.send(JSON.stringify({
                  type: "ice-candidate",
                  candidate: ice.candidate
                }));
              } else {
                localIceq.push(ice.candidate);
              }
            };            
            connection.onicecandidateerror = () => {
              // throw new Error();
            };
            connection.oniceconnectionstatechange = () => {
              console.log("ICE STATE:", connection.iceConnectionState);
            };
            
            connection.onconnectionstatechange = () => {
              console.log("PC STATE:", connection.connectionState);
            };
            
            connection.ontrack = (event) => {
              if (event.receiver) {
                // 最低レイテンシ（0秒バッファ）を要求。ネットワークパケットロス時のカクつきよりもリアルタイム性を優先。
                event.receiver.playoutDelayHint = 0;
              }
              stream.addTrack(event.track);
            };
            
            await connection.setRemoteDescription(new RTCSessionDescription({
              type: "offer",
              sdp: data.sdp
            }));
            for (const c of pendingCandidates) {
              await connection.addIceCandidate(c);
            }
            pendingCandidates = [];
            const answer = await connection.createAnswer();
            await connection.setLocalDescription(answer);
            await new Promise(r => setTimeout(r, 0));
            for (const c of localIceq) {
              wss.send(JSON.stringify({
                type: "ice-candidate",
                candidate: c
              }));
            }
            localIceq = [];
            wss.send(JSON.stringify({
              type: "answer",
              sdp: answer.sdp
            }));
            break;
          }
          case "ice-candidate": {
            if (!data.candidate) break;
          
            const cand = new RTCIceCandidate(data.candidate);
          
            if (connection.remoteDescription) {
              await connection.addIceCandidate(cand);
            } else {
              pendingCandidates.push(cand);
            }
            break;
          }
          default: {
            throw new Error();
            break;
          }
        }
      } catch (e) {
        console.log(e)
        setErr("The service sent invalid format response");
        await reset();
      }
    };

    wss.onclose = async (a) => {
      console.log(a);
      console.log("closed");
      // setErr("Connection refused, your key is wrong or internal server error");
      await reset();
    };
    wss.onerror = async () => {
      setErr("Connection failed");
      await reset();
    };
  };
  const reset = async () => {
    nok = null;
    if (channels) {
      Object.values(channels).forEach(c => {
        try { c.close(); } catch(e) {}
      });
    }
    channels = null;
    if (connection) {
      connection.close();
    }
    connection = null;
    if (stream) {
      stream.getTracks().forEach(t => t.stop());
    }
    stream = new MediaStream();
    setStatus(STATUS.INTERACTIVE);
    wss?.close();
  };

  const getStream = () => stream;

  const sendData = (data) => {
    if (!channels) return;
    const channel = Object.values(channels).find(c => c.readyState === 'open') || Object.values(channels)[0];
    if (channel && channel.readyState === 'open') {
      channel.send(data);
    }
  };

  return { setup, start, reset, getStream, sendData };
}