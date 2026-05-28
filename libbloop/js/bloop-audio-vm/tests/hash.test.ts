import { describe, it, expect } from 'vitest';
import { hashBloop, xorshift32 } from '../src/audio/BloopHash.js';

describe('hashBloop', () => {
  it('is deterministic', () => {
    expect(hashBloop('hello')).toBe(hashBloop('hello'));
    expect(hashBloop('murmur')).toBe(hashBloop('murmur'));
  });

  it('returns an unsigned 32-bit integer', () => {
    const h = hashBloop('test');
    expect(h).toBeGreaterThanOrEqual(0);
    expect(h).toBeLessThanOrEqual(0xffffffff);
    expect(Number.isInteger(h)).toBe(true);
  });

  it('empty string has a stable hash', () => {
    expect(hashBloop('')).toBe(hashBloop(''));
  });

  it('different inputs produce different hashes', () => {
    expect(hashBloop('hello')).not.toBe(hashBloop('world'));
    expect(hashBloop('murmur')).not.toBe(hashBloop('buzz'));
    expect(hashBloop('a')).not.toBe(hashBloop('b'));
  });

  it('a single character difference changes the hash', () => {
    expect(hashBloop('hello')).not.toBe(hashBloop('hellp'));
  });

  it('order matters', () => {
    expect(hashBloop('ab')).not.toBe(hashBloop('ba'));
  });
});

describe('xorshift32', () => {
  it('never returns zero', () => {
    let state = 1;
    for (let i = 0; i < 1000; i++) {
      state = xorshift32(state);
      expect(state).toBeGreaterThan(0);
    }
  });

  it('is deterministic from the same seed', () => {
    const a: number[] = [];
    const b: number[] = [];
    let s1 = 42, s2 = 42;
    for (let i = 0; i < 100; i++) {
      s1 = xorshift32(s1);
      s2 = xorshift32(s2);
      a.push(s1);
      b.push(s2);
    }
    expect(a).toEqual(b);
  });

  it('state 0 is handled without getting stuck', () => {
    const result = xorshift32(0);
    expect(result).toBeGreaterThan(0);
  });
});
