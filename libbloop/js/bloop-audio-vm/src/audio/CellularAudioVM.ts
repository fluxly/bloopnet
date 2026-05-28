import { normalizeBloop } from './BloopNormalizer.js';
import { hashBloop, xorshift32 } from './BloopHash.js';
import { BloopParams, deriveParams, deriveRule } from './rules.js';

export interface CellularAudioVMOptions {
  sampleRate?: number;  // default 44100
  cellCount?: number;   // default 256
  bloop?: string;       // initial bloop string
}

// Triangle wavefold: maps any real number back to [-1, 1] by folding at the boundaries.
function fold(x: number): number {
  x = ((x % 4) + 4) % 4;  // wrap to [0, 4)
  if (x > 2) x = 4 - x;   // fold upper half → [0, 2]
  return x - 1;            // shift to [-1, 1]
}

export class CellularAudioVM {
  private readonly sampleRate: number;
  private readonly cellCount: number;

  // Bloop state — set by setBloop
  private source: string = '';
  private seed: number = 0;
  private rule: number = 30;
  private params!: BloopParams;

  // CA row — two buffers, swapped each generation
  private row: Uint8Array;
  private nextRow: Uint8Array;

  // Counters
  private sampleIndex: number = 0;
  private generation: number = 0;

  // Filter state
  private dcX: number = 0;
  private dcY: number = 0;
  private lpState: number = 0;

  constructor(options: CellularAudioVMOptions = {}) {
    this.sampleRate = options.sampleRate ?? 44100;
    this.cellCount = options.cellCount ?? 256;
    this.row = new Uint8Array(this.cellCount);
    this.nextRow = new Uint8Array(this.cellCount);
    this.setBloop(options.bloop ?? '');
  }

  setBloop(source: string): void {
    this.source = normalizeBloop(source);
    this.seed = hashBloop(this.source);
    this.rule = deriveRule(this.source, this.seed);
    this.params = deriveParams(this.source, this.seed, this.sampleRate);
    this.initRow();
    this.resetCounters();
  }

  reset(): void {
    this.initRow();
    this.resetCounters();
  }

  nextSample(): number {
    const p = this.params;
    const t = this.sampleIndex;

    if (t % p.samplesPerGeneration === 0) {
      this.stepCA();
    }

    // Read offset, optionally vibrato-modulated
    let vibratoOffset = 0;
    if (p.vibrato) {
      vibratoOffset = Math.floor(
        p.vibratoDepth * Math.sin(2 * Math.PI * t / p.vibratoPeriod) * this.cellCount * 0.05
      );
    }

    const rawOffset = t * p.readSpeed + p.phaseOffset + vibratoOffset;
    const offset = ((rawOffset % this.cellCount) + this.cellCount) % this.cellCount;

    let sample = this.readBits(offset);

    // Vowel-driven waveshaping
    sample = this.shapeOscillator(sample, p.oscillatorMode);

    // Consonant-driven transforms
    if (p.bitcrush) sample = Math.round(sample * p.crush) / p.crush;
    if (p.fold)     sample = fold(sample * p.foldAmount);
    if (p.gate)     sample *= ((t >>> p.gateShift) & 1) ? 1 : 0.2;
    if (p.clip)     sample = Math.tanh(sample * p.drive);
    if (p.ring)     sample *= Math.sin(2 * Math.PI * t / p.ringFreq);

    // One-pole lowpass
    if (p.lowpass) {
      this.lpState += p.lowpassAlpha * (sample - this.lpState);
      sample = this.lpState;
    }

    // DC blocker, final gain, hard clip
    sample = this.dcBlock(sample);
    sample = Math.max(-1, Math.min(1, sample * p.gain));

    this.sampleIndex++;
    return sample;
  }

  nextBuffer(output: Float32Array): void {
    for (let i = 0; i < output.length; i++) {
      output[i] = this.nextSample();
    }
  }

  get currentRule(): number  { return this.rule; }
  get currentSeed(): number  { return this.seed; }
  get currentSource(): string { return this.source; }

  // ── Private ──────────────────────────────────────────────────────────────────

  private resetCounters(): void {
    this.sampleIndex = 0;
    this.generation = 0;
    this.dcX = 0;
    this.dcY = 0;
    this.lpState = 0;
  }

  private initRow(): void {
    let state = this.seed === 0 ? 2166136261 : this.seed;
    for (let i = 0; i < this.cellCount; i++) {
      state = xorshift32(state);
      this.row[i] = state & 1;
    }
    // Guard against all-zero row (rule 0 stays dead)
    if (!this.row.some(c => c === 1)) this.row[0] = 1;
  }

  private stepCA(): void {
    const n = this.cellCount;
    const rule = this.rule;
    for (let i = 0; i < n; i++) {
      const pattern = (this.row[(i - 1 + n) % n] << 2)
                    | (this.row[i]              << 1)
                    |  this.row[(i + 1)     % n];
      this.nextRow[i] = (rule >>> pattern) & 1;
    }
    // Swap row buffers in place
    const tmp = this.row;
    this.row = this.nextRow;
    this.nextRow = tmp;
    this.generation++;
  }

  // Read 8 consecutive cells as an integer, map to [-1, 1].
  private readBits(offset: number): number {
    let value = 0;
    for (let i = 0; i < 8; i++) {
      value = (value << 1) | this.row[(offset + i) % this.cellCount];
    }
    return (value / 127.5) - 1;
  }

  // Vowel modes: a=0 e=1 i=2 o=3 u=4 y=5
  private shapeOscillator(x: number, mode: number): number {
    switch (mode) {
      case 0: return Math.sin(x * Math.PI * 0.5);   // a — sine-ish
      case 1: return 1 - 2 * Math.abs(x);            // e — triangle
      case 2: return x >= 0 ? 1 : -1;                // i — square/pulse
      case 3: return x;                               // o — saw (passthrough)
      case 4: return x * (1 + Math.abs(x));           // u — noise texture
      case 5: return Math.sin(2 * Math.PI * x);       // y — FM phase-warp
      default: return x;
    }
  }

  private dcBlock(x: number): number {
    const y = x - this.dcX + 0.995 * this.dcY;
    this.dcX = x;
    this.dcY = y;
    return y;
  }
}
