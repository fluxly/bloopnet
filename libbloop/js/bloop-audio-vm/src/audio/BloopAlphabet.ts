// Vowels determine oscillator / readout mode.
// Order here maps directly to oscillatorMode indices (0–5).
export const VOWELS = 'aeiouy';

export function countChar(source: string, char: string): number {
  let count = 0;
  for (const c of source) if (c === char) count++;
  return count;
}

// Returns the index (0–5) of the most frequent vowel in source.
// 0=a 1=e 2=i 3=o 4=u 5=y
export function dominantVowelMode(source: string): number {
  let maxCount = 0;
  let mode = 0;
  for (let i = 0; i < VOWELS.length; i++) {
    const n = countChar(source, VOWELS[i]);
    if (n > maxCount) { maxCount = n; mode = i; }
  }
  return mode;
}
