// Valid characters for the audio VM (lowercase a-z, space, pipe, question, hyphen).
// Newline is a first-class Bloop text symbol but is not part of the audio alphabet.
const VALID_CHARS = new Set('abcdefghijklmnopqrstuvwxyz |?-');

export const MAX_BLOOP_LENGTH = 64;

export function normalizeBloop(source: string): string {
  let result = '';
  const lower = source.toLowerCase();
  for (let i = 0; i < lower.length && result.length < MAX_BLOOP_LENGTH; i++) {
    if (VALID_CHARS.has(lower[i])) result += lower[i];
  }
  return result;
}
