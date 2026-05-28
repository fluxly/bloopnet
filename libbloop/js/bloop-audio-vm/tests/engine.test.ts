import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { BloopAudioEngine } from '../src/audio/BloopAudioEngine.js';

// ── AudioContext mock ─────────────────────────────────────────────────────────

function makeMockAudioContext(state: AudioContextState = 'running') {
  const gainParam = {
    value: 1,
    setValueAtTime: vi.fn(),
    linearRampToValueAtTime: vi.fn(),
    cancelScheduledValues: vi.fn(),
    setTargetAtTime: vi.fn(),
  };

  const gainNode = {
    gain: gainParam,
    connect: vi.fn(),
    disconnect: vi.fn(),
  };

  const processor = {
    onaudioprocess: null as ((e: any) => void) | null,
    connect: vi.fn(),
    disconnect: vi.fn(),
  };

  const ctx = {
    state,
    sampleRate: 44100,
    currentTime: 0,
    destination: {},
    createGain: vi.fn(() => gainNode),
    createScriptProcessor: vi.fn(() => processor),
    resume: vi.fn(() => Promise.resolve()),
    close: vi.fn(() => Promise.resolve()),
    // Expose internals so tests can inspect/trigger them.
    _gainNode: gainNode,
    _processor: processor,
    _gainParam: gainParam,
  };

  return ctx;
}

type MockCtx = ReturnType<typeof makeMockAudioContext>;

// Stub AudioContext globally so the engine can call `new AudioContext()`.
function stubAudioContext(ctx: MockCtx) {
  vi.stubGlobal('AudioContext', vi.fn(() => ctx));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function fireAudioProcess(ctx: MockCtx, bufferSize = 2048) {
  const outputData = new Float32Array(bufferSize);
  ctx._processor.onaudioprocess?.({
    outputBuffer: { getChannelData: (_: number) => outputData },
  });
  return outputData;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('BloopAudioEngine — construction', () => {
  it('default volume is 0.5', () => {
    const engine = new BloopAudioEngine();
    expect(engine.volume).toBe(0.5);
  });

  it('default bloop is empty string', () => {
    const engine = new BloopAudioEngine();
    expect(engine.bloop).toBe('');
  });

  it('playing starts as false', () => {
    const engine = new BloopAudioEngine();
    expect(engine.playing).toBe(false);
  });

  it('accepts initial bloop and volume', () => {
    const engine = new BloopAudioEngine({ bloop: 'murmur', volume: 0.8 });
    expect(engine.bloop).toBe('murmur');
    expect(engine.volume).toBe(0.8);
  });
});

describe('BloopAudioEngine — configuration', () => {
  it('setBloop updates the bloop property', () => {
    const engine = new BloopAudioEngine();
    engine.setBloop('buzz');
    expect(engine.bloop).toBe('buzz');
  });

  it('volume setter clamps negative values to 0', () => {
    const engine = new BloopAudioEngine();
    engine.volume = -0.5;
    expect(engine.volume).toBe(0);
  });

  it('volume setter clamps values above 1 to 1', () => {
    const engine = new BloopAudioEngine();
    engine.volume = 2;
    expect(engine.volume).toBe(1);
  });

  it('volume constructor option is clamped', () => {
    expect(new BloopAudioEngine({ volume: -1 }).volume).toBe(0);
    expect(new BloopAudioEngine({ volume: 5 }).volume).toBe(1);
  });
});

describe('BloopAudioEngine — audio lifecycle', () => {
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
  });

  it('play() creates an AudioContext', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    expect(AudioContext).toHaveBeenCalledOnce();
  });

  it('play() creates a gain node and processor, and connects them', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    expect(mockCtx.createGain).toHaveBeenCalledOnce();
    expect(mockCtx.createScriptProcessor).toHaveBeenCalledOnce();
    expect(mockCtx._processor.connect).toHaveBeenCalledWith(mockCtx._gainNode);
    expect(mockCtx._gainNode.connect).toHaveBeenCalledWith(mockCtx.destination);
  });

  it('play() fades in from 0 to target volume', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello', volume: 0.6 });
    await engine.play();
    expect(mockCtx._gainParam.setValueAtTime).toHaveBeenCalledWith(0, 0);
    expect(mockCtx._gainParam.linearRampToValueAtTime).toHaveBeenCalledWith(
      0.6,
      BloopAudioEngine.FADE_S,
    );
  });

  it('play() sets playing to true', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    expect(engine.playing).toBe(true);
  });

  it('calling play() twice is a no-op on the second call', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    await engine.play();
    expect(mockCtx.createGain).toHaveBeenCalledOnce();
  });

  it('play() resumes a suspended AudioContext', async () => {
    mockCtx = makeMockAudioContext('suspended');
    stubAudioContext(mockCtx);
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    expect(mockCtx.resume).toHaveBeenCalledOnce();
  });

  it('play() does not resume a running AudioContext', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    expect(mockCtx.resume).not.toHaveBeenCalled();
  });

  it('stop() sets playing to false immediately', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    engine.stop();
    expect(engine.playing).toBe(false);
  });

  it('stop() schedules a gain fade-out', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello', volume: 0.5 });
    await engine.play();
    engine.stop();
    expect(mockCtx._gainParam.linearRampToValueAtTime).toHaveBeenLastCalledWith(
      0,
      BloopAudioEngine.FADE_S,
    );
  });

  it('stop() disconnects the graph after the fade', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    engine.stop();
    // Nothing disconnected yet — fade is in progress.
    expect(mockCtx._processor.disconnect).not.toHaveBeenCalled();
    // Advance past the fade + buffer.
    vi.advanceTimersByTime(BloopAudioEngine.FADE_S * 1000 + 20);
    expect(mockCtx._processor.disconnect).toHaveBeenCalledOnce();
    expect(mockCtx._gainNode.disconnect).toHaveBeenCalledOnce();
  });

  it('stop() when not playing is a no-op', () => {
    const engine = new BloopAudioEngine();
    expect(() => engine.stop()).not.toThrow();
  });

  it('onaudioprocess fills the output buffer with samples', async () => {
    const engine = new BloopAudioEngine({ bloop: 'murmur' });
    await engine.play();
    const outputData = fireAudioProcess(mockCtx);
    const nonZero = outputData.filter(s => s !== 0).length;
    expect(nonZero).toBeGreaterThan(0);
  });

  it('all onaudioprocess samples are within [-1, 1]', async () => {
    const engine = new BloopAudioEngine({ bloop: 'buzz' });
    await engine.play();
    const outputData = fireAudioProcess(mockCtx);
    for (const s of outputData) {
      expect(s).toBeGreaterThanOrEqual(-1);
      expect(s).toBeLessThanOrEqual(1);
    }
  });

  it('volume change while playing updates the gain node', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    engine.volume = 0.3;
    expect(mockCtx._gainParam.setTargetAtTime).toHaveBeenCalledWith(0.3, 0, 0.01);
  });

  it('volume change while not playing does not touch gain node', () => {
    const engine = new BloopAudioEngine();
    engine.volume = 0.3;
    expect(mockCtx.createGain).not.toHaveBeenCalled();
  });

  it('dispose() closes the AudioContext after the fade', async () => {
    const engine = new BloopAudioEngine({ bloop: 'hello' });
    await engine.play();
    engine.dispose();
    vi.advanceTimersByTime(BloopAudioEngine.FADE_S * 1000 + 100);
    expect(mockCtx.close).toHaveBeenCalledOnce();
  });

  it('setBloop while playing reseeds the VM (new bloop sounds different)', async () => {
    // Run two engines: one keeps the same bloop, one changes it.
    const unchanged = new BloopAudioEngine({ bloop: 'murmur' });
    const changed = new BloopAudioEngine({ bloop: 'murmur' });

    const mockCtx2 = makeMockAudioContext('running');
    vi.stubGlobal('AudioContext', vi.fn()
      .mockReturnValueOnce(mockCtx)
      .mockReturnValueOnce(mockCtx2));

    await unchanged.play();
    await changed.play();

    const before = fireAudioProcess(mockCtx2).slice();
    changed.setBloop('buzz');
    const after = fireAudioProcess(mockCtx2);

    const same = before.every((v, i) => v === after[i]);
    expect(same).toBe(false);
  });
});

describe('BloopAudioEngine — throws without Web Audio', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('play() throws if AudioContext is not available', async () => {
    vi.stubGlobal('AudioContext', undefined);
    const engine = new BloopAudioEngine();
    await expect(engine.play()).rejects.toThrow('Web Audio API');
  });
});
