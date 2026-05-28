// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Register the element before any tests run.
import '../src/bloop-audio-vm.js';
import { BloopAudioElement } from '../src/components/BloopAudioElement.js';

// ── AudioContext mock (same shape as engine tests) ────────────────────────────

function makeMockAudioContext(state: AudioContextState = 'running') {
  const gainParam = {
    value: 1,
    setValueAtTime: vi.fn(),
    linearRampToValueAtTime: vi.fn(),
    cancelScheduledValues: vi.fn(),
    setTargetAtTime: vi.fn(),
  };
  const gainNode = { gain: gainParam, connect: vi.fn(), disconnect: vi.fn() };
  const processor = {
    onaudioprocess: null as ((e: unknown) => void) | null,
    connect: vi.fn(),
    disconnect: vi.fn(),
  };
  return {
    state,
    sampleRate: 44100,
    currentTime: 0,
    destination: {},
    createGain: vi.fn(() => gainNode),
    createScriptProcessor: vi.fn(() => processor),
    resume: vi.fn(() => Promise.resolve()),
    close: vi.fn(() => Promise.resolve()),
    _gainNode: gainNode,
    _processor: processor,
  };
}

type MockCtx = ReturnType<typeof makeMockAudioContext>;

function stubAudioContext(ctx: MockCtx) {
  vi.stubGlobal('AudioContext', vi.fn(() => ctx));
}

// Flush microtasks (needed after async play() triggered by autoplay).
const flushMicrotasks = () => new Promise<void>(resolve => setTimeout(resolve, 0));

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeElement(attrs: Record<string, string> = {}): BloopAudioElement {
  const el = document.createElement('bloop-audio-vm') as BloopAudioElement;
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
  return el;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('Registration', () => {
  it('registers as "bloop-audio-vm"', () => {
    expect(customElements.get('bloop-audio-vm')).toBe(BloopAudioElement);
  });

  it('document.createElement returns a BloopAudioElement', () => {
    const el = document.createElement('bloop-audio-vm');
    expect(el).toBeInstanceOf(BloopAudioElement);
  });

  it('observedAttributes includes bloop, volume, autoplay, rate, preview', () => {
    const obs = BloopAudioElement.observedAttributes;
    expect(obs).toContain('bloop');
    expect(obs).toContain('volume');
    expect(obs).toContain('autoplay');
    expect(obs).toContain('rate');
    expect(obs).toContain('preview');
  });
});

describe('Default property values', () => {
  it('bloop defaults to empty string', () => {
    expect(makeElement().bloop).toBe('');
  });

  it('volume defaults to 0.5', () => {
    expect(makeElement().volume).toBe(0.5);
  });

  it('rate defaults to 1', () => {
    expect(makeElement().rate).toBe(1);
  });

  it('playing defaults to false', () => {
    expect(makeElement().playing).toBe(false);
  });
});

describe('Reflected properties', () => {
  it('setting bloop property sets the attribute', () => {
    const el = makeElement();
    el.bloop = 'murmur';
    expect(el.getAttribute('bloop')).toBe('murmur');
  });

  it('setting bloop attribute updates the property', () => {
    const el = makeElement();
    el.setAttribute('bloop', 'buzz');
    expect(el.bloop).toBe('buzz');
  });

  it('setting volume property updates the attribute', () => {
    const el = makeElement();
    el.volume = 0.7;
    expect(el.getAttribute('volume')).toBe('0.7');
  });

  it('volume attribute is parsed back to a number', () => {
    const el = makeElement({ volume: '0.3' });
    expect(el.volume).toBe(0.3);
  });

  it('setting rate property updates the attribute', () => {
    const el = makeElement();
    el.rate = 2;
    expect(el.getAttribute('rate')).toBe('2');
  });

  it('invalid volume attribute falls back to 0.5', () => {
    const el = makeElement({ volume: 'bad' });
    expect(el.volume).toBe(0.5);
  });
});

describe('bloop-audio-change event', () => {
  it('fires when the bloop attribute changes', () => {
    const el = makeElement();
    const handler = vi.fn();
    el.addEventListener('bloop-audio-change', handler);
    el.bloop = 'murmur';
    expect(handler).toHaveBeenCalledOnce();
  });

  it('detail.bloop is the normalized bloop string', () => {
    const el = makeElement();
    const handler = vi.fn();
    el.addEventListener('bloop-audio-change', handler);
    el.bloop = 'HELLO!';
    const detail = (handler.mock.calls[0][0] as CustomEvent).detail;
    expect(detail.bloop).toBe('hello');
  });

  it('detail.seed is a non-negative integer', () => {
    const el = makeElement();
    const handler = vi.fn();
    el.addEventListener('bloop-audio-change', handler);
    el.bloop = 'murmur';
    const detail = (handler.mock.calls[0][0] as CustomEvent).detail;
    expect(detail.seed).toBeGreaterThanOrEqual(0);
    expect(Number.isInteger(detail.seed)).toBe(true);
  });

  it('fires when setting bloop via property setter', () => {
    const el = makeElement();
    const handler = vi.fn();
    el.addEventListener('bloop-audio-change', handler);
    el.setAttribute('bloop', 'buzz');
    expect(handler).toHaveBeenCalledOnce();
  });

  it('event bubbles', () => {
    const el = makeElement();
    document.body.append(el);
    const handler = vi.fn();
    document.body.addEventListener('bloop-audio-change', handler);
    el.bloop = 'hello';
    expect(handler).toHaveBeenCalledOnce();
    el.remove();
    document.body.removeEventListener('bloop-audio-change', handler);
  });
});

describe('play / stop lifecycle', () => {
  let mockCtx: MockCtx;

  beforeEach(() => {
    vi.useFakeTimers();
    mockCtx = makeMockAudioContext('running');
    stubAudioContext(mockCtx);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    document.body.innerHTML = '';
  });

  it('play() sets playing to true', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    expect(el.playing).toBe(true);
  });

  it('play() sets the playing attribute', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    expect(el.hasAttribute('playing')).toBe(true);
  });

  it('play() dispatches bloop-audio-start', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    const handler = vi.fn();
    el.addEventListener('bloop-audio-start', handler);
    await el.play();
    expect(handler).toHaveBeenCalledOnce();
    expect((handler.mock.calls[0][0] as CustomEvent).detail.bloop).toBe('hello');
  });

  it('stop() sets playing to false', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    el.stop();
    expect(el.playing).toBe(false);
  });

  it('stop() removes the playing attribute', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    el.stop();
    expect(el.hasAttribute('playing')).toBe(false);
  });

  it('stop() dispatches bloop-audio-stop', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    const handler = vi.fn();
    el.addEventListener('bloop-audio-stop', handler);
    el.stop();
    expect(handler).toHaveBeenCalledOnce();
    expect((handler.mock.calls[0][0] as CustomEvent).detail.bloop).toBe('hello');
  });

  it('stop() without play() does not throw', () => {
    const el = makeElement();
    expect(() => el.stop()).not.toThrow();
  });

  it('volume change while playing reaches the engine gain node', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    el.volume = 0.3;
    expect(mockCtx._gainNode.gain.setTargetAtTime).toHaveBeenCalledWith(0.3, 0, 0.01);
  });
});

describe('autoplay attribute', () => {
  let mockCtx: MockCtx;

  beforeEach(() => {
    mockCtx = makeMockAudioContext('running');
    stubAudioContext(mockCtx);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    document.body.innerHTML = '';
  });

  it('autoplay triggers play() on connectedCallback', async () => {
    const el = makeElement({ bloop: 'hello', autoplay: '' });
    document.body.append(el);
    await flushMicrotasks();
    expect(el.playing).toBe(true);
  });

  it('without autoplay, connecting does not start playback', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await flushMicrotasks();
    expect(el.playing).toBe(false);
  });
});

describe('error handling', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    document.body.innerHTML = '';
  });

  it('play() dispatches bloop-audio-error when AudioContext is unavailable', async () => {
    vi.stubGlobal('AudioContext', undefined);
    const el = makeElement();
    document.body.append(el);
    const handler = vi.fn();
    el.addEventListener('bloop-audio-error', handler);
    await expect(el.play()).rejects.toThrow();
    expect(handler).toHaveBeenCalledOnce();
    expect((handler.mock.calls[0][0] as CustomEvent).detail.error).toBeDefined();
  });

  it('bloop-audio-error event bubbles', async () => {
    vi.stubGlobal('AudioContext', undefined);
    const el = makeElement();
    document.body.append(el);
    const handler = vi.fn();
    document.body.addEventListener('bloop-audio-error', handler);
    await expect(el.play()).rejects.toThrow();
    expect(handler).toHaveBeenCalledOnce();
    document.body.removeEventListener('bloop-audio-error', handler);
  });
});

describe('disconnectedCallback', () => {
  let mockCtx: MockCtx;

  beforeEach(() => {
    vi.useFakeTimers();
    mockCtx = makeMockAudioContext('running');
    stubAudioContext(mockCtx);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    document.body.innerHTML = '';
  });

  it('removing the element stops playback', async () => {
    const el = makeElement({ bloop: 'hello' });
    document.body.append(el);
    await el.play();
    el.remove();
    expect(el.playing).toBe(false);
  });

  it('removing the element without playing does not throw', () => {
    const el = makeElement();
    document.body.append(el);
    expect(() => el.remove()).not.toThrow();
  });
});
