/**
 * WebCrypto APIを使用した安全な暗号化関数（X25519）
 */

// X25519のサポート状態とフォールバックモジュール
let supportsX25519 = null; // null = 未チェック, true/false = チェック済み
let x25519Module = null;

/**
 * X25519のサポートをチェックし、必要に応じてフォールバックを初期化
 */
async function initializeX25519() {
  if (supportsX25519 !== null) {
    return; // 既に初期化済み
  }

  // WebCrypto APIでX25519がサポートされているかテスト
  try {
    const testKey = await crypto.subtle.generateKey(
      { name: "X25519" },
      false,
      ["deriveBits", "deriveKey"]
    ).catch(() => null);
    supportsX25519 = testKey !== null;
  } catch {
    supportsX25519 = false;
  }

  // X25519がサポートされていない場合のフォールバック用
  if (!supportsX25519) {
    try {
      // 動的インポートで@noble/curvesを使用
      const { x25519 } = await import("@noble/curves/ed25519.js");
      x25519Module = x25519;
    } catch (e) {
      console.warn("X25519 fallback library not available:", e);
    }
  }
}

/**
 * WebCrypto APIで安全な乱数を生成（32バイト）
 * @returns {Promise<Uint8Array>}
 */
async function generateSecureRandomBytes(length) {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return array;
}

/**
 * Diffie-Hellman 鍵ペア生成（X25519 - 32バイト）
 * @returns {Promise<{privateKey: Uint8Array | CryptoKey, publicKey: Uint8Array | CryptoKey}>}
 */
export async function createDHPair() {
  await initializeX25519();
  
  if (supportsX25519) {
    // WebCrypto APIでX25519を使用
    const keyPair = await crypto.subtle.generateKey(
      {
        name: "X25519",
      },
      true, // extractable
      ["deriveBits", "deriveKey"]
    );

    return {
      privateKey: keyPair.privateKey,
      publicKey: keyPair.publicKey,
    };
  } else if (x25519Module) {
    // フォールバック: @noble/curvesを使用（WebCrypto APIの乱数生成を使用）
    const privateKey = await generateSecureRandomBytes(32);
    const publicKey = x25519Module.getPublicKey(privateKey);

    return {
      privateKey,
      publicKey,
    };
  } else {
    throw new Error("X25519 is not supported and fallback library is not available");
  }
}

/**
 * 公開鍵をUint8Array形式でエクスポート（32バイト）
 * @param {CryptoKey | Uint8Array} publicKey
 * @returns {Promise<Uint8Array>}
 */
export async function exportPublicKey(publicKey) {
  if (publicKey instanceof CryptoKey) {
    const exported = await crypto.subtle.exportKey("raw", publicKey);
    const exportedArray = new Uint8Array(exported);
    // X25519の公開鍵は32バイトである必要がある
    if (exportedArray.length !== 32) {
      throw new Error(`Invalid X25519 public key length: ${exportedArray.length}, expected 32`);
    }
    return exportedArray;
  } else if (publicKey instanceof Uint8Array) {
    if (publicKey.length !== 32) {
      throw new Error(`Invalid X25519 public key length: ${publicKey.length}, expected 32`);
    }
    return publicKey;
  } else {
    throw new TypeError("publicKey must be CryptoKey or Uint8Array");
  }
}

/**
 * 公開鍵をCryptoKey形式でインポート（32バイト）
 * @param {Uint8Array} publicKeyBytes
 * @returns {Promise<CryptoKey | Uint8Array>}
 */
export async function importPublicKey(publicKeyBytes) {
  await initializeX25519();
  
  if (!(publicKeyBytes instanceof Uint8Array)) {
    throw new TypeError("publicKeyBytes must be Uint8Array");
  }
  
  if (publicKeyBytes.length !== 32) {
    throw new Error(`Invalid X25519 public key length: ${publicKeyBytes.length}, expected 32`);
  }

  if (supportsX25519) {
    return await crypto.subtle.importKey(
      "raw",
      publicKeyBytes,
      {
        name: "X25519",
      },
      true,
      []
    );
  } else {
    // フォールバック: Uint8Arrayのまま返す
    return publicKeyBytes;
  }
}

/**
 * 共通秘密の生成（X25519 - 32バイト）
 * @param {CryptoKey | Uint8Array} privateKey
 * @param {CryptoKey | Uint8Array} publicKey
 * @returns {Promise<Uint8Array>}
 */
export async function createSharedSecret(privateKey, publicKey) {
  await initializeX25519();
  
  if (supportsX25519 && privateKey instanceof CryptoKey && publicKey instanceof CryptoKey) {
    // WebCrypto APIを使用
    const sharedSecret = await crypto.subtle.deriveBits(
      {
        name: "X25519",
        public: publicKey,
      },
      privateKey,
      256 // 256 bits = 32 bytes
    );

    const result = new Uint8Array(sharedSecret);
    if (result.length !== 32) {
      throw new Error(`Invalid shared secret length: ${result.length}, expected 32`);
    }
    return result;
  } else if (x25519Module && privateKey instanceof Uint8Array && publicKey instanceof Uint8Array) {
    // フォールバック: @noble/curvesを使用
    if (privateKey.length !== 32 || publicKey.length !== 32) {
      throw new Error(`Invalid key length: private=${privateKey.length}, public=${publicKey.length}, expected 32`);
    }
    const sharedSecret = x25519Module.getSharedSecret(privateKey, publicKey);
    if (sharedSecret.length !== 32) {
      throw new Error(`Invalid shared secret length: ${sharedSecret.length}, expected 32`);
    }
    return sharedSecret;
  } else {
    throw new TypeError("Keys must be CryptoKey (with X25519 support) or Uint8Array (with fallback)");
  }
}

/**
 * 共通秘密 → ハッシュ（Netover 用）
 * @param {Uint8Array} byteArr sharedSecret
 * @returns {Promise<string>} hex 文字列（比較・ID 用）
 */
export async function getHash(byteArr) {
  if (!(byteArr instanceof Uint8Array)) {
    throw new TypeError("getHash expects Uint8Array");
  }

  // プロトコル文脈（用途バインド）
  const context = new TextEncoder().encode("Netover v1 shared-secret hash");

  // 結合: sharedSecret + context
  const combined = new Uint8Array(byteArr.length + context.length);
  combined.set(byteArr, 0);
  combined.set(context, byteArr.length);

  // WebCrypto APIでSHA-256ハッシュ計算
  const hashBuffer = await crypto.subtle.digest("SHA-256", combined);
  const hashBytes = new Uint8Array(hashBuffer);

  // Uint8Array → hex 文字列
  return bytesToHex(hashBytes);
}

/**
 * Uint8Array → hex
 */
function bytesToHex(bytes) {
  return Array.from(bytes, (b) =>
    b.toString(16).padStart(2, "0")
  ).join("");
}

export async function hkdf(sharedSecret, salt, info, length = 32) {
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    sharedSecret,
    "HKDF",
    false,
    ["deriveBits", "deriveKey"]
  );

  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: "HKDF",
      hash: "SHA-256",
      salt: salt,
      info: info
    },
    keyMaterial,
    length * 8
  );

  return new Uint8Array(derivedBits);
}

export function bytesToBase64(bytes) {
  let binary = "";
  for (let b of bytes) {
    binary += String.fromCharCode(b);
  }
  return btoa(binary);
}

// base64 → Uint8Array
export function base64ToBytes(b64) {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * HMAC署名を作る関数
 * @param {Uint8Array} key - HMAC鍵
 * @param {Uint8Array} nonce - 任意の長さ（ここでは32バイト）のデータ
 * @returns {Promise<Uint8Array>} - HMAC署名
 */
export async function hmacSign(key, nonce) {
  // CryptoKey 化
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "HMAC", hash: { name: "SHA-256" } },
    false,
    ["sign"]
  );

  // ArrayBuffer or ArrayBufferView であることを保証する
  let nonceBuf;
  if (nonce instanceof Uint8Array || ArrayBuffer.isView(nonce)) {
    nonceBuf = nonce.buffer instanceof ArrayBuffer && nonce.byteOffset === 0 && nonce.byteLength === nonce.buffer.byteLength
      ? nonce
      : new Uint8Array(nonce);
  } else if (nonce instanceof ArrayBuffer) {
    nonceBuf = new Uint8Array(nonce);
  } else {
    throw new TypeError("Nonce must be an ArrayBuffer or ArrayBufferView");
  }

  const signature = await crypto.subtle.sign(
    "HMAC",
    cryptoKey,
    nonceBuf
  );
  return new Uint8Array(signature);
}

export function concatSorted(a, b) {
  // Uint8Array 用の辞書順比較
  let sortOrder = 0;
  const len = Math.min(a.length, b.length);
  for (let i = 0; i < len; i++) {
    if (a[i] < b[i]) {
      sortOrder = -1;
      break;
    }
    if (a[i] > b[i]) {
      sortOrder = 1;
      break;
    }
  }
  if (sortOrder === 0) {
    sortOrder = a.length - b.length;
  }

  const first = sortOrder < 0 ? a : b;
  const second = sortOrder < 0 ? b : a;

  // 結合
  const res = new Uint8Array(first.length + second.length);
  res.set(first, 0);
  res.set(second, first.length);
  return res;
}

export async function sha256(data) {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", data));
}