import { CellularAudioVM } from './CellularAudioVM.js';

export interface BloopAudioEngineOptions {
  bloop?: string;
  volume?: number;  // 0–1, default 0.5
}

export class BloopAudioEngine {
  // ScriptProcessorNode is deprecated but is the simplest synchronous prototype path.
  // Upgrade to AudioWorkletNode in a later phase.
  static readonly BUFFER_SIZE = 2048;
  static readonly FADE_S = 0.02;  // 20ms fade in/out

  private vm: CellularAudioVM;
  private _bloop: string;
  private _volume: number;
  private _playing = false;

  private audioCtx: AudioContext | null = null;
  private gainNode: GainNode | null = null;
  private processor: ScriptProcessorNode | null = null;

  constructor(options: BloopAudioEngineOptions = {}) {
    this._bloop = options.bloop ?? '';
    this._volume = Math.max(0, Math.min(1, options.volume ?? 0.5));
    // Pre-create VM at default sample rate; it will be replaced at the correct
    // rate once AudioContext is available.
    this.vm = new CellularAudioVM({ bloop: this._bloop });
  }

  async play(): Promise<void> {
    if (this._playing) return;

    if (typeof AudioContext === 'undefined') {
      throw new Error('BloopAudioEngine requires the Web Audio API');
    }

    if (!this.audioCtx) {
      this.audioCtx = new AudioContext();
    }
    if (this.audioCtx.state === 'suspended') {
      await this.audioCtx.resume();
    }

    // Re-create VM at the AudioContext's actual sample rate.
    this.vm = new CellularAudioVM({ bloop: this._bloop, sampleRate: this.audioCtx.sampleRate });

    // Gain node — starts at 0 and ramps up for a click-free fade in.
    this.gainNode = this.audioCtx.createGain();
    this.gainNode.gain.setValueAtTime(0, this.audioCtx.currentTime);
    this.gainNode.gain.linearRampToValueAtTime(
      this._volume,
      this.audioCtx.currentTime + BloopAudioEngine.FADE_S,
    );

    this.processor = this.audioCtx.createScriptProcessor(BloopAudioEngine.BUFFER_SIZE, 0, 1);
    this.processor.onaudioprocess = (event: AudioProcessingEvent) => {
      this.vm.nextBuffer(event.outputBuffer.getChannelData(0));
    };

    this.processor.connect(this.gainNode);
    this.gainNode.connect(this.audioCtx.destination);

    this._playing = true;
  }

  stop(): void {
    if (!this._playing || !this.audioCtx || !this.gainNode || !this.processor) return;

    // Fade out before disconnecting to avoid clicks.
    const now = this.audioCtx.currentTime;
    this.gainNode.gain.cancelScheduledValues(now);
    this.gainNode.gain.setValueAtTime(this.gainNode.gain.value, now);
    this.gainNode.gain.linearRampToValueAtTime(0, now + BloopAudioEngine.FADE_S);

    const fadeDoneMs = BloopAudioEngine.FADE_S * 1000 + 10;
    const proc = this.processor;
    const gain = this.gainNode;
    setTimeout(() => { proc.disconnect(); gain.disconnect(); }, fadeDoneMs);

    this.processor = null;
    this.gainNode = null;
    this._playing = false;
  }

  reset(): void {
    const wasPlaying = this._playing;
    if (wasPlaying) this.stop();
    this.vm.reset();
    if (wasPlaying) {
      // Wait for the fade-out to finish before restarting.
      setTimeout(() => { void this.play(); }, BloopAudioEngine.FADE_S * 1000 + 10);
    }
  }

  setBloop(source: string): void {
    this._bloop = source;
    if (this.audioCtx && this._playing) {
      // Hard reset to new bloop (Phase 1 behaviour; crossfade comes in Phase 5).
      this.vm = new CellularAudioVM({ bloop: source, sampleRate: this.audioCtx.sampleRate });
    } else {
      this.vm.setBloop(source);
    }
  }

  get volume(): number { return this._volume; }
  set volume(v: number) {
    this._volume = Math.max(0, Math.min(1, v));
    if (this.gainNode && this.audioCtx && this._playing) {
      // Smooth ramp to avoid zipper noise.
      this.gainNode.gain.setTargetAtTime(this._volume, this.audioCtx.currentTime, 0.01);
    }
  }

  get playing(): boolean { return this._playing; }
  get bloop(): string { return this._bloop; }

  dispose(): void {
    this.stop();
    const ctx = this.audioCtx;
    this.audioCtx = null;
    setTimeout(() => { void ctx?.close(); }, BloopAudioEngine.FADE_S * 1000 + 50);
  }
}
