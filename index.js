import express from 'express';
import expressWs from 'express-ws';
import { remotes, matchesQueue, generateRandomID, targets } from "./components/shared.js";
import { v4 as uuidv4 } from 'uuid';
import { errorCodes } from './components/error.js';

const app = express();

expressWs(app);

app.use(express.static("public"));

// CORS middleware
app.use((req, res, next) => {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
  if (req.method === 'OPTIONS') {
    return res.sendStatus(200);
  }
  next();
});

function checkBothReady(localws, remotews) {
  if (localws.options.host === "local" && remotews.options.host === "remote") {
    if (localws.options.accepted && remotews.options.accepted) {
      return true;
    } else {
      return false;
    }
  } else {
    throw new Error("Invalid host");
  }
};

app.ws('/ws/pairing/remote', (ws, req) => {
  console.log("From remote", req);
  console.log("From remote", req);
  const id = generateRandomID();
  matchesQueue.remote.set(id, ws);
  ws.options = {
    id: id,
    ready: false,
    host: "remote",
    accepted: false,
    local_id: null
  }
  ws.send(JSON.stringify({
    type: "rand-id",
    id: id,
  }));
  // let local_ws = null;

  ws.on('message', (msg) => {
    console.log("From remote", msg);
    console.log("From remote", msg);
    try {
      const data = JSON.parse(msg);
      switch (data.type) {
        case "init_remote": {
          // すでにInitializeされていたらclose
          if (ws.options.ready) {
            ws.close(errorCodes.ALREADY_INITIALIZED);
            return;
          }
          // 準備完了
          ws.options.ready = true;
          // 公開鍵セット
          ws.remote_pub = data.pub;
          break;
        }
        case "accept": { // 接続を許可するとき
          const local_ws = matchesQueue.local.get(ws.options.local_id);
          // 準備もしくは該当先が設定されていなければclose
          if (!ws.options.ready || !local_ws) {
            ws.close(errorCodes.NOT_READY);
            return;
          }
          // 接続許可
          ws.options.accepted = true;
          // 該当先に接続許可したことを送信
          local_ws.send(JSON.stringify({
            type: "accept_from_remote",
          }));
          // 両方Acceptなら
          if (checkBothReady(local_ws, ws)) {
            ws.send(JSON.stringify({
              type: "allowed_key"
            }));
            local_ws.send(JSON.stringify({
              type: "allowed_key"
            }));
          }
          break;
        }
        case "deny": {
          console.log("FFFFFF");
          const local_ws = matchesQueue.local.get(ws.options.local_id);
          // 準備もしくは該当先が設定されていなければclose
          if (!ws.options.ready || !local_ws) {
            ws.close(errorCodes.NOT_READY);
            return;
          }
          // localの情報を消去
          local_ws.send(JSON.stringify({ type: "deny_from_remote" }));
          matchesQueue.local.delete(ws.options.local_id);
          ws.options.local_id = null;
          // 該当先に接続拒否したことを送信
          break;
        }
        case "cancel": {
          const local_ws = matchesQueue.local.get(ws.options.local_id);
          if (!local_ws) {
            ws.close(errorCodes.NOT_FOUND);
            return;
          }
          ws.options.accepted = false;
          local_ws.send(JSON.stringify({
            type: "cancel_from_remote",
          }));
          break;
        }
        default: {
          ws.close(errorCodes.INVALID_PAYLOAD);
          break;
        }
      }
    } catch (error) {
      console.error(error);
      ws.close(errorCodes.INTERNAL_SERVER_ERROR);
    }
  });

  ws.on('close', () => {
    const local_ws = matchesQueue.local.get(ws.options.local_id);
    if (local_ws) {
      local_ws.send(JSON.stringify({
        type: "disconnected_from_remote"
      }));
    }
    matchesQueue.remote.delete(ws.options.id);
    console.log(`Match ${ws.options.id} disconnected`);
  });

  ws.on('error', (error) => {
    console.error(error);
  });
});
app.ws('/ws/pairing/local', (ws, req) => {
  // ランダムにIDを生成し、セット
  const id = generateRandomID();
  matchesQueue.local.set(id, ws);
  // プロパティを設定
  ws.options = {
    id: id, // 自分自身のID
    ready: false, // 準備完了フラグ
    host: "local",
    accepted: false, // 接続許可フラグ
    remote_id: null // 該当先のID
  }
  let remote_ws = null;
  ws.on('message', (msg) => {
    console.log("From local", msg);
    console.log("From local", msg);
    try {
      const data = JSON.parse(msg);
      switch (data.type) {
        case "init_local": {
          // すでに初期化されていたらclose
          if (ws.options.ready) {
            ws.close(errorCodes.ALREADY_INITIALIZED);
            return;
          }
          // 準備完了
          ws.options.ready = true;
          // 公開鍵セット
          ws.local_pub = data.pub;
          // 該当先のIDセット
          ws.options.remote_id = data.remote_id;
          remote_ws = matchesQueue.remote.get(ws.options.remote_id);
          // 該当先が存在しない場合はclose
          if (!remote_ws) {
            ws.close(errorCodes.NOT_FOUND);
            return;
          }
          // すでに該当先が接続中であったならclose
          if (remote_ws.options.local_id) {
            ws.close(errorCodes.ALREADY_MACHING);
            return;
          }
          // 該当先に公開鍵を送信
          remote_ws.send(JSON.stringify({
            type: "exchange_from_local",
            pub: ws.local_pub, // 自分の公開鍵
            local_id: ws.options.id, // 自分のID
          }));
          // 自分に該当先の公開鍵を送信
          ws.send(JSON.stringify({
            type: "exchange_from_remote",
            pub: remote_ws.remote_pub, // 該当先の公開鍵
            remote_id: ws.options.remote_id // 該当先のID
          }));
          remote_ws.options.local_id = ws.options.id;
          break;

          // このコードにより、「ws.options.ready == true なら remote_wsはnullでない」ことが保証される。
        }
        case "accept": {
          // 該当先を取得
          // 準備もしくは該当先が設定されていなければclose
          if (!ws.options.ready || !ws.options.remote_id) {
            ws.close(errorCodes.NOT_READY);
            return;
          }
          // 接続許可
          ws.options.accepted = true;
          // 該当先に接続許可したことを送信
          remote_ws.send(JSON.stringify({
            type: "accept_from_local",
          }));
          // もし同時にAcceptしてたら
          if (checkBothReady(ws, remote_ws)) {
            ws.send(JSON.stringify({
              type: "allowed_key",
            }));
            remote_ws.send(JSON.stringify({
              type: "allowed_key",
            }));
          }
          break;
        }
        case "deny": {
          // 該当先を取得
          // 準備もしくは該当先が設定されていなければclose
          if (!ws.options.ready || !ws.options.remote_id) {
            ws.close(errorCodes.NOT_READY);
            return;
          }
          // 該当先に接続拒否したことを送信
          remote_ws.send(JSON.stringify({
            type: "deny_from_local",
          }));
          break;
        }
        case "cancel": {
          if (!ws.options.ready || !ws.options.remote_id) {
            ws.close(errorCodes.NOT_READY);
            return;
          }
          ws.options.accepted = false;
          // 該当先にキャンセルしたことを送信
          remote_ws.send(JSON.stringify({
            type: "cancel_from_local",
          }));
          break;
        }
        default: {
          ws.close(errorCodes.INVALID_PAYLOAD);
          break;
        }
      }
    } catch (error) {
      console.error(error);
      ws.close(errorCodes.INTERNAL_SERVER_ERROR);
    }
  });

  ws.on('close', () => {
    if (remote_ws) {
      remote_ws.send(JSON.stringify({
        type: "disconnected_from_local"
      }));
    }
    if (ws.options && ws.options.id) {
      matchesQueue.local.delete(ws.options.id);
    }
    console.log(`Match ${ws.options.id} disconnected`);
  });

  ws.on('error', (error) => {
    console.error(error);
  });
});

// HTTP - Local Pairing
// app.get("/pairing/local", (req, res) => {
//   try {
//     const { remote_id, pub } = req.query;
//     if (!remote_id || !pub) {
//       return res.status(400).json({ error: "Public key is required" });
//     }
//     const remote_pub = matchesQueue.remote.get(remote_id).remote_pub;
//     if (!remote_pub) {
//       return res.status(400).json({ error: "Remote public key not found" });
//     }
//     return res.json({ remote_pub: remote_pub });
//   } catch (error) {
//     return res.status(500).json({ error: "Internal server error" });
//   }
// });
app.ws('/ws/launch/target', (ws, req) => {
  let keys = [];
  ws.iceq = [];
  ws.options = {
    ready: false,
    controller: null
  };

  ws.on('message', (msg) => {
    try {
      const data = JSON.parse(msg);
      console.log("From target", data);
      switch (data.type) {
        case "init": {
          // data.keysが配列であることを確認
          if (!data.keys || !Array.isArray(data.keys)) {
            ws.close(errorCodes.INVALID_REQUEST);
            return;
          }
          // 長さが0以上1729以下であることを確認
          if (data.keys.length < 0 || data.keys.length > 1729) {
            ws.close(errorCodes.INVALID_REQUEST);
            return;
          }
          // すべての要素が文字列であることを確認
          if (!data.keys.every(key => typeof key === 'string')) {
            ws.close(errorCodes.INVALID_REQUEST);
            return;
          }

          // すべての条件を満たしたのでkeysを更新
          keys = data.keys;
          for (const key of keys) {
            targets.set(key, ws);
          }

          ws.ready = true;
          ws.send(JSON.stringify({
            type: "init_success"
          }));

          break;
        }
        case "queryverify": {
          // console.log(data.nonce);
          if (!Array.isArray(data.nonce)) throw new Error();
          ws?.options.controller.send(JSON.stringify({
            type: "queryverify",
            nonce: data.nonce
          }));
          break;
        }
        case "accept-offer": {
          ws?.options.controller.send(JSON.stringify({
            type: "accept-offer",
            sdp: data.sdp
          }));
          if (ws.options.controller?.iceq?.length > 0) {
            for (const c of ws.options.controller.iceq) {
              ws.send(JSON.stringify({
                type: "ice-candidate",
                candidate: c
              }));
            }
            ws.options.controller.iceq = [];
          }
          break;
        }
        case "ice-candidate": {
          if (ws.options.controller) {
            ws?.options.controller.send(JSON.stringify({
              type: "ice-candidate",
              candidate: data.candidate
            }));
          } else {
            ws.iceq.push(data.candidate);
          }
          break;
        }
        case "deny": {
          ws?.options.controller.send(JSON.stringify({
            type: "deny",
          }));
          break;
        }
        default: {
          ws.close(errorCodes.INVALID_REQUEST);
          break;
        }
      }
    } catch (e) {
      // keysが配列であることを確認してから処理
      ws.close(errorCodes.INVALID_REQUEST);
    }
  });
  ws.on("close", () => {
    if (Array.isArray(keys)) {
      for (const key of keys) {
        targets.delete(key);
      }
    }
  });
});
app.ws('/ws/launch/controller', (ws, req) => {
  let target = null;
  ws.iceq = [];
  ws.on("message", (msg) => {
    try {
      const data = JSON.parse(msg);
      console.log("From controller", data);
      switch (data.type) {
        case "request": {
          console.log(data);
          if (!data.id) throw new Error();
          target = targets.get(data.id);
          // console.log(target);
          if (!target) {
            // No target found, close this ws
            ws.close(errorCodes.INVALID_REQUEST);
            return;
          }
          if (target.options.controller) {
            ws.close(errorCodes.ALREADY_MACHING);
            return;
          }
          target.options.controller = ws;
          if (target.iceq?.length > 0) {
            for (const candidate of target.iceq) {
              target.send({
                type: "ice-candidate",
                candidate
              });
            }
            target.iceq = [];
          }
          target.send(JSON.stringify({
            type: "requested",
            keyid: data.id
          }));
          break;
        }
        case "sendsign": {
          if (!target) {
            ws.close(errorCodes.INVALID_REQUEST);
            return;
          }
          target.send(JSON.stringify({
            type: "getsign",
            sign: data.sign
          }));
          break;
        }
        case "answer": {
          try {
            target.send(JSON.stringify({
              type: "answer",
              answer: data.sdp
            }));
          } catch (e) {
            console.log(e);
            throw e;
          }
          break;
        }
        case "ice-candidate": {
          if (target.options.controller) {
            target.send(JSON.stringify({
              type: "ice-candidate",
              candidate: data.candidate
            }));
          } else {
            ws.iceq.push(data.candidate);
          }
          break;
        }
        default: {
          ws.close(errorCodes.INVALID_REQUEST);
          return;
        }
      }
    } catch (e) {
      console.log(e);
      ws.close(errorCodes.INVALID_REQUEST);
    }
  });

  ws.on('close', () => {
    // ターゲットに対するコントローラーは一意に定まるかnullである
    // 自分自身にターゲットを持っていないならそのまま閉じる
    if (target) {
      console.log("controller disconnected");
      target.options.controller = null;
      target.options.ready = false;
      // targetにcontrollerが切断したことを通知し、再接続を受け入れる状態に戻す
      try {
        target.send(JSON.stringify({
          type: "controller_disconnected"
        }));
      } catch (e) {
        console.error("Failed to notify target of controller disconnect:", e);
      }
    }
  });
});

const port = process.env.PORT || 3001;
app.listen(port, () => {
  console.log(`Server is running on port ${port}`);
});