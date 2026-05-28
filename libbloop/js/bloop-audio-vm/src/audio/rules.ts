import { countChar, dominantVowelMode } from './BloopAlphabet.js';

export interface BloopParams {
  // CA timing
  samplesPerGeneration: number;
  readSpeed: number;         // row read offset advances this many cells per sample
  phaseOffset: number;       // initial read position in the row

  // Output
  gain: number;
  oscillatorMode: number;    // 0–5, driven by dominant vowel

  // Transforms — consonant-driven
  bitcrush: boolean;
  crush: number;             // quantization steps; fewer = harsher

  fold: boolean;
  foldAmount: number;        // pre-fold gain

  gate: boolean;
  gateShift: number;         // rhythmic gating via bit-shift of sampleIndex

  clip: boolean;
  drive: number;             // tanh drive amount

  lowpass: boolean;
  lowpassAlpha: number;      // one-pole coeff; lower = stronger smoothing

  ring: boolean;
  ringFreq: number;          // ring-mod LFO period in samples

  vibrato: boolean;
  vibratoDepth: number;      // fraction of cellCount to modulate read offset
  vibratoPeriod: number;     // samples per vibrato cycle
}

// Derive an elementary CA rule (0–255) from the bloop hash, biased by certain consonants.
export function deriveRule(source: string, seed: number): number {
  let rule = seed & 0xff;
  // Character biasing per spec
  if (source.includes('x')) rule ^= 0b01011010;  // x: XOR-heavy transform
  if (source.includes('l')) rule &= 0b11101110;  // l: smooth out chaotic bits
  if (source.includes('z')) rule |= 0b00100100;  // z: add resonant edge patterns
  return rule & 0xff;
}

// Derive all synthesis parameters from the normalized source string and its hash.
export function deriveParams(source: string, seed: number, sampleRate: number): BloopParams {
  const samplesPerGeneration = Math.max(1, Math.floor(sampleRate / 8000));
  const readSpeed = 1 + (seed & 0x3);
  const phaseOffset = (seed >>> 4) & 0xff;

  const bCount = countChar(source, 'b');
  const cCount = countChar(source, 'c');
  const fCount = countChar(source, 'f');
  const gCount = countChar(source, 'g');
  const lCount = countChar(source, 'l');
  const rCount = countChar(source, 'r');
  const vCount = countChar(source, 'v');

  return {
    samplesPerGeneration,
    readSpeed,
    phaseOffset,
    gain: 0.25,
    oscillatorMode: dominantVowelMode(source),

    bitcrush: bCount > 0,
    crush: Math.max(2, 16 - bCount * 2),

    fold: fCount > 0,
    foldAmount: 1.0 + fCount * 0.5,

    gate: gCount > 0,
    gateShift: Math.max(2, 5 - gCount),

    clip: cCount > 0,
    drive: 1.0 + cCount * 0.5,

    lowpass: lCount > 0,
    lowpassAlpha: Math.max(0.05, 1.0 - lCount * 0.15),

    ring: rCount > 0,
    ringFreq: Math.max(16, 64 + rCount * 16),

    vibrato: vCount > 0,
    vibratoDepth: Math.min(0.5, vCount * 0.1),
    vibratoPeriod: Math.max(20, 100 - vCount * 10),
  };
}
