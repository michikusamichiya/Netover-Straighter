// Shared variables and functions
const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

export const remotes = new Map();
export const matchesQueue = { remote: new Map(), local: new Map() };
export const targets = new Map();

export function generateRandomID() {
  let it = "";
  for (let i = 0; i < 6; i++) {
    it += characters.charAt(Math.floor(Math.random() * characters.length));
  }
  return it;
}
