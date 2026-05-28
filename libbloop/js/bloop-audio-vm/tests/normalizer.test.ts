import { describe, it, expect } from 'vitest';
import { normalizeBloop, MAX_BLOOP_LENGTH } from '../src/audio/BloopNormalizer.js';

describe('normalizeBloop', () => {
  it('passes through valid lowercase', () => {
    expect(normalizeBloop('hello world')).toBe('hello world');
  });

  it('lowercases uppercase letters', () => {
    expect(normalizeBloop('Hello World')).toBe('hello world');
  });

  it('strips digits', () => {
    expect(normalizeBloop('abc123')).toBe('abc');
  });

  it('strips unsupported punctuation', () => {
    expect(normalizeBloop('hello!')).toBe('hello');
    expect(normalizeBloop('foo.bar')).toBe('foobar');
    expect(normalizeBloop('a+b=c')).toBe('abc');
  });

  it('preserves pipe, question mark, and hyphen', () => {
    expect(normalizeBloop('a|b?c-d')).toBe('a|b?c-d');
  });

  it('preserves spaces', () => {
    expect(normalizeBloop('a b c')).toBe('a b c');
  });

  it('strips newlines', () => {
    expect(normalizeBloop('hello\nworld')).toBe('helloworld');
  });

  it('caps output at MAX_BLOOP_LENGTH', () => {
    const result = normalizeBloop('a'.repeat(200));
    expect(result.length).toBe(MAX_BLOOP_LENGTH);
  });

  it('handles empty string', () => {
    expect(normalizeBloop('')).toBe('');
  });

  it('handles all-invalid input', () => {
    expect(normalizeBloop('123!@#')).toBe('');
  });

  it('MAX_BLOOP_LENGTH is 64', () => {
    expect(MAX_BLOOP_LENGTH).toBe(64);
  });
});
