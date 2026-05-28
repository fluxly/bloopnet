// FNV-1a 32-bit hash. Returns an unsigned integer.
export function hashBloop(source: string): number {
  let hash = 2166136261;
  for (let i = 0; i < source.length; i++) {
    hash ^= source.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

// xorshift32 PRNG — used to seed the initial CA row from the bloop hash.
export function xorshift32(state: number): number {
  if (state === 0) state = 1;
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
