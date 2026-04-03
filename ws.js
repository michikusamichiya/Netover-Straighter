const express = require('express');
const expressWs = require('express-ws');
const router = express.Router();
const {
  remotes,
  matchesQueue,
  generateRandomID,
} = require("./shared.js");

// Remote Pairing
router.ws('/pairing/remote', (ws, req) => {
  const id = generateRandomID();
  matchesQueue.set(id, ws);
  ws.options = {
    id: id,
    ready: false,
  }
  ws.send(JSON.stringify({
    type: "rand-id",
    id: id,
  }));

  console.log("New client connected");

  ws.on('message', (msg) => {
    try {
      const data = JSON.parse(msg);
      switch (data.type) {
        case "init_remote": {
          ws.options.ready = true;
          ws.remote_pub = data.pub;
          break;
        }
        default: {
          ws.close();
          break;
        }
      }
    } catch (error) {
      console.error(error);
      ws.close();
    }
  });

  ws.on('close', () => {
    matchesQueue.delete(ws.options.id);
    console.log(`Match ${ws.options.id} disconnected`);
  });

  ws.on('error', (error) => {
    console.error(error);
  });
});
router.ws('/pairing/local', (ws, req) => {
  ws.on('message', (msg) => {
    try {
      const data = JSON.parse(msg);
      switch (data.type) {
        case "init_local": {
          ws.options = {
            ready: false,
          }
          if (!ws.options) {
            console.error("Fuck");
            ws.close();
            return;
          }
          ws.options.ready = true;
          ws.local_pub = data.pub;

          const remote_id = data.remote_id;
          const remote_ws = matchesQueue.get(remote_id);
          if (!remote_ws) {
            ws.close();
            return;
          }
          const remote_pub = remote_ws.remote_pub;
          if (!remote_ws.options.ready) {
            ws.close();
            return;
          }
          remote_ws.send(JSON.stringify({
            type: "exchange_from_local",
            pub: data.pub
          }));
          ws.send(JSON.stringify({
            type: "exchange_from_remote",
            pub: remote_pub,
          }));
          break;
        }
        default: {
          ws.close();
          break;
        }
      }
    } catch (error) {
      console.error(error);
      ws.close();
    }
  });

  ws.on('close', () => {
    if (ws.options && ws.options.id) {
      matchesQueue.delete(ws.options.id);
      console.log(`Match ${ws.options.id} disconnected`);
    }
  });

  ws.on('error', (error) => {
    console.error(error);
  });
});

module.exports = router;
