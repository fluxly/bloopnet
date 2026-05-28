import { BloopAudioEngine } from '../audio/BloopAudioEngine.js';
import { hashBloop } from '../audio/BloopHash.js';
import { normalizeBloop } from '../audio/BloopNormalizer.js';

export class BloopAudioElement extends HTMLElement {
  static readonly observedAttributes = ['bloop', 'volume', 'autoplay', 'rate', 'preview'];

  private readonly engine: BloopAudioEngine;

  constructor() {
    super();
    this.engine = new BloopAudioEngine();
  }

  // ── Lifecycle ──────────────────────────────────────────────────────────────

  connectedCallback(): void {
    if (this.hasAttribute('autoplay')) {
      void this.play().catch(() => {
        // Error event already dispatched inside play()
      });
    }
  }

  disconnectedCallback(): void {
    this.engine.dispose();
  }

  attributeChangedCallback(name: string, _old: string | null, value: string | null): void {
    switch (name) {
      case 'bloop': {
        const normalized = normalizeBloop(value ?? '');
        this.engine.setBloop(normalized);
        this.dispatchEvent(new CustomEvent('bloop-audio-change', {
          bubbles: true,
          detail: { bloop: normalized, seed: hashBloop(normalized) },
        }));
        break;
      }
      case 'volume': {
        const v = parseFloat(value ?? '');
        if (!isNaN(v)) this.engine.volume = v;
        break;
      }
      case 'rate':
        // Rate is stored as an attribute for Phase 5; not yet wired to the VM.
        break;
      case 'autoplay':
      case 'preview':
        // Boolean flags — presence is all that matters, no value handling needed.
        break;
    }
  }

  // ── Reflected properties ───────────────────────────────────────────────────

  get bloop(): string { return this.getAttribute('bloop') ?? ''; }
  set bloop(v: string) { this.setAttribute('bloop', v); }

  get volume(): number {
    const v = parseFloat(this.getAttribute('volume') ?? '');
    return isNaN(v) ? 0.5 : v;
  }
  set volume(v: number) { this.setAttribute('volume', String(v)); }

  get rate(): number {
    const v = parseFloat(this.getAttribute('rate') ?? '');
    return isNaN(v) ? 1 : v;
  }
  set rate(v: number) { this.setAttribute('rate', String(v)); }

  // playing is read-only; the attribute is set/removed by play() and stop()
  // for CSS hook: bloop-audio-vm[playing] { ... }
  get playing(): boolean { return this.engine.playing; }

  // ── Methods ────────────────────────────────────────────────────────────────

  async play(): Promise<void> {
    try {
      await this.engine.play();
      this.setAttribute('playing', '');
      this.dispatchEvent(new CustomEvent('bloop-audio-start', {
        bubbles: true,
        detail: { bloop: this.bloop },
      }));
    } catch (err) {
      this.dispatchEvent(new CustomEvent('bloop-audio-error', {
        bubbles: true,
        detail: { error: err },
      }));
      throw err;
    }
  }

  stop(): void {
    this.engine.stop();
    this.removeAttribute('playing');
    this.dispatchEvent(new CustomEvent('bloop-audio-stop', {
      bubbles: true,
      detail: { bloop: this.bloop },
    }));
  }

  reset(): void {
    this.engine.reset();
  }
}
