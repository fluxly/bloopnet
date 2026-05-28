import { describe, it, expect } from 'vitest';
import { CellularAudioVM } from '../src/audio/CellularAudioVM.js';

// Hash a Float32Array to a stable integer for snapshot-style comparison.
function sampleHash(samples: Float32Array): number {
  const dv = new DataView(new ArrayBuffer(4));
  let h = 2166136261;
  for (let i = 0; i < samples.length; i++) {
    dv.setFloat32(0, samples[i]);
    h ^= dv.getUint32(0);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function collect(vm: CellularAudioVM, n: number): Float32Array {
  const buf = new Float32Array(n);
  vm.nextBuffer(buf);
  return buf;
}

describe('CellularAudioVM — basic operation', () => {
  it('generates samples without throwing', () => {
    const vm = new CellularAudioVM({ bloop: 'hello world' });
    expect(() => vm.nextSample()).not.toThrow();
  });

  it('all samples are within [-1, 1]', () => {
    const vm = new CellularAudioVM({ bloop: 'buzz' });
    for (let i = 0; i < 1024; i++) {
      const s = vm.nextSample();
      expect(s).toBeGreaterThanOrEqual(-1);
      expect(s).toBeLessThanOrEqual(1);
    }
  });

  it('nextBuffer fills the array', () => {
    const vm = new CellularAudioVM({ bloop: 'murmur' });
    const buf = new Float32Array(256);
    vm.nextBuffer(buf);
    const nonZero = buf.filter(s => s !== 0).length;
    expect(nonZero).toBeGreaterThan(0);
  });

  it('does not crash on empty bloop', () => {
    const vm = new CellularAudioVM({ bloop: '' });
    expect(() => collect(vm, 256)).not.toThrow();
  });
});

describe('CellularAudioVM — determinism', () => {
  it('same bloop produces identical first 1024 samples', () => {
    const a = new CellularAudioVM({ bloop: 'murmur', sampleRate: 44100 });
    const b = new CellularAudioVM({ bloop: 'murmur', sampleRate: 44100 });
    expect(collect(a, 1024)).toEqual(collect(b, 1024));
  });

  it('different bloops produce different sample hashes', () => {
    const bloops = ['murmur', 'buzz', 'aeiuollll', 'txxxtxxxtxxx', 'ktt-ktt-ktt'];
    const hashes = bloops.map(bloop => {
      const vm = new CellularAudioVM({ bloop, sampleRate: 44100 });
      return sampleHash(collect(vm, 1024));
    });
    const unique = new Set(hashes);
    expect(unique.size).toBe(bloops.length);
  });

  it('uppercase input sounds the same as lowercase', () => {
    const lower = new CellularAudioVM({ bloop: 'hello', sampleRate: 44100 });
    const upper = new CellularAudioVM({ bloop: 'HELLO', sampleRate: 44100 });
    expect(collect(lower, 256)).toEqual(collect(upper, 256));
  });

  it('unsupported characters are stripped before seeding', () => {
    const clean = new CellularAudioVM({ bloop: 'abc', sampleRate: 44100 });
    const dirty = new CellularAudioVM({ bloop: 'a1b2c3', sampleRate: 44100 });
    expect(collect(clean, 256)).toEqual(collect(dirty, 256));
  });
});

describe('CellularAudioVM — reset and setBloop', () => {
  it('reset reproduces the same samples from the start', () => {
    const vm = new CellularAudioVM({ bloop: 'hello', sampleRate: 44100 });
    const first = collect(vm, 512);
    vm.reset();
    const second = collect(vm, 512);
    expect(first).toEqual(second);
  });

  it('setBloop changes the sound', () => {
    const vm = new CellularAudioVM({ bloop: 'murmur', sampleRate: 44100 });
    const before = sampleHash(collect(vm, 1024));
    vm.setBloop('buzz');
    const after = sampleHash(collect(vm, 1024));
    expect(before).not.toBe(after);
  });

  it('setBloop followed by reset reproduces setBloop output', () => {
    const vm = new CellularAudioVM({ bloop: 'buzz', sampleRate: 44100 });
    const first = collect(vm, 256);
    vm.reset();
    const second = collect(vm, 256);
    expect(first).toEqual(second);
  });
});

describe('CellularAudioVM — example bloops from spec', () => {
  const examples = [
    'murmur',
    'buzz',
    'aeiuollll',
    'txxxtxxxtxxx',
    'ktt-ktt-ktt',
    'little bells|soft static|deep hum',
    'bloopnet sings itself',
  ];

  for (const bloop of examples) {
    it(`does not crash or clip: "${bloop}"`, () => {
      const vm = new CellularAudioVM({ bloop, sampleRate: 44100 });
      const buf = new Float32Array(2048);
      vm.nextBuffer(buf);
      for (const s of buf) {
        expect(s).toBeGreaterThanOrEqual(-1);
        expect(s).toBeLessThanOrEqual(1);
      }
    });
  }
});

describe('CellularAudioVM — currentSource / currentSeed / currentRule', () => {
  it('exposes normalized source', () => {
    const vm = new CellularAudioVM({ bloop: 'Hello World!' });
    expect(vm.currentSource).toBe('hello world');
  });

  it('exposes a non-zero seed for non-empty input', () => {
    const vm = new CellularAudioVM({ bloop: 'murmur' });
    expect(vm.currentSeed).toBeGreaterThan(0);
  });

  it('exposes a rule in 0–255', () => {
    const vm = new CellularAudioVM({ bloop: 'buzz' });
    expect(vm.currentRule).toBeGreaterThanOrEqual(0);
    expect(vm.currentRule).toBeLessThanOrEqual(255);
  });
});
