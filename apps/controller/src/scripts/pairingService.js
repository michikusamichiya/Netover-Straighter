import { createDHPair, createSharedSecret, getHash, exportPublicKey, importPublicKey, hkdf, bytesToBase64, sha256, concatSorted } from "./crypto";
import { downloadText } from "./tools.js";
import { getSettings } from "./settings.js";

export const STATUS = {
  INTERACTIVE: "interactive",
  GENERATING: "generating",
  CONNECTING: "connecting",
  EXCHANGING: "exchanging",
  WAITING_FOR_CHECK: "waitingforcheck",
  CONNECTION_REFUSED: "refused",
  DENIED: "denied",
  COMPLETE: "complete"
};

export const STATUS_TEXT = {
  [STATUS.INTERACTIVE]: "",
  [STATUS.GENERATING]: "Generating authentication keys...",
  [STATUS.CONNECTING]: "Connecting to server...",
  [STATUS.EXCHANGING]: "Exchanging keys...",
  [STATUS.WAITING_FOR_CHECK]: "Waiting for check...",
  [STATUS.CONNECTION_REFUSED]: "Connection refused",
  [STATUS.DENIED]: "You or the target denied the pairing.",
  [STATUS.COMPLETE]: "Complete"
};

export function validatePairingId(id) {
  return /[A-Z]{6}/.test(id);
}

export function createPairingService({
  onStatusChange,
  onError,
  setCheckhash,
  setAccepted,
  setAccept
} = {}) {
  /**
   * @type { WebSocket | null }
   */
  let wss = null;
  /**
   * @type { { publicKey: CryptoKey | Uint8Array | null, privateKey: CryptoKey | Uint8Array | null } }
   */
  let keypair = { publicKey: null, privateKey: null };
  let accepted = -1;
  let accept = false;
  let remoteId = null;
  let remotePub = null;

  const close = () => {
    try {
      wss?.close();
      onError?.("Connection closed");
    } finally {
      wss = null;
    }
  };

  const fail = (msg = "Failed or canceled") => {
    onError?.(msg);
    onStatusChange?.(STATUS.INTERACTIVE);
    close();
  };

  const start = async (id) => {
    remoteId = id;
    if (!validatePairingId(id)) {
      onError?.("Invalid Format!");
      return;
    }

    try {
      onStatusChange?.(STATUS.GENERATING);
      const pair = await createDHPair();
      keypair = pair;

      onStatusChange?.(STATUS.CONNECTING);
      const publicKeyBytes = await exportPublicKey(pair.publicKey);
      const publicKeyArray = Array.from(publicKeyBytes);

      wss = new WebSocket(`${getSettings().serverUrl}/ws/pairing/local`);

      wss.onopen = async () => {
        try {
          onStatusChange?.(STATUS.EXCHANGING);
          wss?.send(JSON.stringify({
            type: "init_local",
            pub: publicKeyArray,
            remote_id: id 
          }));
        } catch (e) {
          console.error(e);
          fail("Failed to send initialization message");
        }
      };

      wss.onmessage = async (msg) => {
        try {
          const data = JSON.parse(msg.data);
          console.log(data);
          if (data.type == "exchange_from_remote") {
            onStatusChange?.(STATUS.WAITING_FOR_CHECK);
            const uremote_pub_bytes = Uint8Array.from(data.pub);
            
            // X25519の公開鍵は32バイトである必要がある
            if (uremote_pub_bytes.length !== 32) {
              throw new Error(`Invalid remote public key length: ${uremote_pub_bytes.length}, expected 32 bytes for X25519`);
            }
            remotePub = uremote_pub_bytes;
            
            const remotePublicKey = await importPublicKey(uremote_pub_bytes);
            keypair = { publicKey: remotePublicKey, privateKey: keypair.privateKey };
            
            const sharedSecret = await createSharedSecret(keypair.privateKey, remotePublicKey);
            const hash = await getHash(sharedSecret);
            setCheckhash(hash.slice(0, 16));
          }
          if (data.type == "accept_from_remote") {
            setAccepted(1);
            accepted = 1;
          }
          if (data.type == "deny_from_remote") {
            setAccepted(0);
            onStatusChange(STATUS.DENIED);
            accepted = 0;
          }
          if (data.type == "cancel_from_remote") {
            setAccepted(-1);
            accepted = -1;
          }
          if (data.type == "disconnected_from_remote") {
            onStatusChange(STATUS.CONNECTION_REFUSED);
            onError(STATUS.CONNECTION_REFUSED);
            close();
          }
          if (data.type == "allowed_key") {
            const sharedSec = await createSharedSecret(keypair.privateKey, keypair.publicKey);
            console.log((await getHash(sharedSec)).slice(0, 16));

            const salt = await sha256(concatSorted(remotePub, publicKeyBytes));
            const info = new TextEncoder().encode("netover-hmac-key-v1");

            const res = await hkdf(sharedSec, salt, info);

            const base64str = bytesToBase64(res);
            downloadText(`${remoteId}_private.nok`, `${remoteId}
${base64str}
\n\n
*********************************************************************************************************************************************************************************\n
WARNING! Do not share this key with others. If it is leaked, it could be used to control the paired computer. If it is leaked, immediately disable the key in Target settings. \n
*********************************************************************************************************************************************************************************
`);

            onStatusChange?.(STATUS.COMPLETE);
          }
        } catch(e) {
          console.error(e);
          fail("Failed to process message: " + e.message);
        }
      };

      wss.onerror = () => fail();
      wss.onclose = () => close();
    } catch (e) {
      console.error(e);
      fail("Failed to start pairing: " + e.message);
    }
  };

  const handleAccept = async () => {
    setAccept(true);
    wss.send(JSON.stringify({
      type: "accept"
    }));
  };
  const handleDeny = async () => {
    onStatusChange(STATUS.DENIED);
    wss.send(JSON.stringify({
      type: "deny"
    }));
  };
  const handleCancel = async () => {
    setAccept(false);
    wss.send(JSON.stringify({
      type: "cancel"
    }));
  };

  return { start, close, handleAccept, handleDeny, handleCancel };
}
