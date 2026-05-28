# Bloopnet Cellular Audio VM — Claude Code Handoff

## Project Goal

Create an embeddable **Bloopnet Cellular Audio Virtual Machine** implemented in **TypeScript** as a **Web Component**.

The component should accept a Bloopnet string, interpret it as a tiny deterministic cellular-audio organism, and play sound in real time using the Web Audio API.

The intended first integration target is the existing **Bloop web terminal example**, where a bloop can be heard as it is being written.

The core idea is inspired by bytebeat, but instead of parsing math expressions like:

```c
t * (t >> 5 | t >> 8)
```

Bloopnet audio treats the bloop itself as a compact seed/program for a deterministic cellular automata audio machine.

Each bloop is simultaneously:

- a text packet
- a tiny score
- a synthesis patch
- a deterministic sound organism
- an atomic audio unit of Bloopnet culture

---

## Constraints

Bloopnet core characters are limited to:

```text
abcdefghijklmnopqrstuvwxyz | ? -
```

That gives:

- 26 lowercase letters
- space
- pipe `|`
- question mark `?`
- hyphen `-`

Total: 30 core symbols.

The implementation should reject, ignore, or normalize characters outside this set depending on configuration.

Suggested default behavior:

- lowercase all input
- strip unsupported characters
- preserve spaces, `|`, `?`, and `-`
- cap bloops at the existing Bloopnet character limit, likely 64 characters

---

## Conceptual Model

Traditional bytebeat usually computes one sample from an expression involving time `t`.

Bloopnet audio should instead work like this:

```text
bloop string -> deterministic cellular automata seed -> evolving bit field -> audio stream
```

The bloop initializes a compact cellular automaton. The automaton evolves over time. Each generation is converted into audio samples.

This gives several advantages:

1. Tiny source strings can produce complex sound.
2. Random-looking output can remain deterministic.
3. Every bloop has a stable sonic identity.
4. The implementation can be compact enough for browser, embedded, and radio-adjacent contexts.
5. It avoids needing a full expression parser.

---

## Recommended First Architecture

Use a layered design:

```text
<bloop-audio-vm> Web Component
        ↓
BloopAudioEngine
        ↓
CellularAudioVM
        ↓
Web Audio Worklet or ScriptProcessor fallback
```

### Phase 1 Recommendation

For fastest implementation, start with a simple main-thread Web Audio implementation using `AudioWorkletNode` if practical, or `ScriptProcessorNode` as a temporary prototype fallback.

However, structure the code so the DSP core is independent:

```ts
class CellularAudioVM {
  constructor(options: CellularAudioVMOptions)
  setBloop(source: string): void
  reset(): void
  nextSample(): number
  nextBuffer(output: Float32Array): void
}
```

The VM should not depend directly on DOM APIs.

---

## Package Shape

Suggested source structure:

```text
src/
  index.ts
  bloop-audio-vm.ts
  audio/
    BloopAudioEngine.ts
    CellularAudioVM.ts
    BloopNormalizer.ts
    BloopHash.ts
    BloopAlphabet.ts
    BloopVoice.ts
    rules.ts
  components/
    BloopAudioElement.ts
  examples/
    terminal-audio-demo.ts
```

If the existing Bloopnet library has its own structure, adapt these names to match existing conventions.

---

## Public Web Component

Create a custom element:

```html
<bloop-audio-vm bloop="murmur|little-bells?" autoplay></bloop-audio-vm>
```

### Attributes

Recommended attributes:

| Attribute | Type | Purpose |
|---|---:|---|
| `bloop` | string | The source bloop to sonify |
| `autoplay` | boolean | Start playback after first user gesture or when allowed |
| `playing` | boolean | Reflects playback state |
| `volume` | number | 0 to 1 output gain |
| `rate` | number | VM tick/sample-rate multiplier |
| `mode` | string | Optional sound mode, e.g. `cell`, `byte`, `drone` |
| `normalize` | boolean | Whether to normalize unsupported characters |
| `preview` | boolean | Whether updates should recompile live while typing |

### Properties

```ts
interface BloopAudioElement extends HTMLElement {
  bloop: string;
  volume: number;
  rate: number;
  playing: boolean;
  play(): Promise<void>;
  stop(): void;
  reset(): void;
}
```

### Events

```ts
bloop-audio-start
bloop-audio-stop
bloop-audio-change
bloop-audio-error
```

Example:

```ts
element.addEventListener('bloop-audio-change', (event) => {
  console.log(event.detail.bloop, event.detail.seed);
});
```

---

## Core VM Design

The VM should use a one-dimensional cellular automaton with a compact row of cells.

Recommended first defaults:

```ts
const DEFAULT_CELL_COUNT = 256;
const DEFAULT_SAMPLE_RATE = 44100;
const DEFAULT_VM_RATE = 1;
```

The automaton maintains:

```ts
interface CellularAudioState {
  row: Uint8Array;
  nextRow: Uint8Array;
  generation: number;
  sampleIndex: number;
  seed: number;
  rule: number;
  phase: number;
  accumulator: number;
  filter: number;
  lastSample: number;
}
```

Each cell can initially be binary: `0` or `1`.

Later, this can expand to 2-bit or 4-bit cell states, but binary is easier to reason about and debug.

---

## Bloop to Seed

A bloop should deterministically produce:

- initial row state
- cellular automata rule
- timbre parameters
- rhythm parameters
- voice layering
- pseudo-random seed

Use a small deterministic hash such as FNV-1a, xorshift, or a simple custom Bloopnet hash.

Example:

```ts
function hashBloop(source: string): number {
  let hash = 2166136261;
  for (let i = 0; i < source.length; i++) {
    hash ^= source.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}
```

Do not use `Math.random()` for core sound generation.

Every bloop must sound the same every time.

---

## Character Semantics

The language should not be a conventional parser at first. Treat each character as a symbolic mutation of the automaton and synthesis state.

### Character Classes

#### Vowels: oscillator / interpretation modes

| Char | Suggested Meaning |
|---|---|
| `a` | smooth sine-ish readout |
| `e` | triangle-ish readout |
| `i` | pulse / square readout |
| `o` | saw / ramp readout |
| `u` | noise-texture readout |
| `y` | FM / phase-warp readout |

#### Consonants: transformations

| Char | Suggested Meaning |
|---|---|
| `b` | bitcrush / reduce resolution |
| `c` | clip / saturate |
| `d` | divide / slow evolution |
| `f` | fold / wavefold |
| `g` | gate / rhythmic mask |
| `h` | hold / sample-and-hold |
| `j` | jump / perturb phase |
| `k` | kick / transient emphasis |
| `l` | lowpass / smooth |
| `m` | multiply / gain / modulation depth |
| `n` | inject deterministic noise |
| `p` | phase shift |
| `q` | quantize |
| `r` | ring modulation |
| `s` | sync / reset phase |
| `t` | time fold / bytebeat-like time influence |
| `v` | vibrato |
| `w` | wrap / modulo shaping |
| `x` | xor / harsh digital transform |
| `z` | resonance / edge emphasis |

#### Structural characters

| Char | Suggested Meaning |
|---|---|
| space | rest / silence / breath / separator |
| `|` | layer split; creates multiple voices mixed together |
| `?` | deterministic probabilistic branch / mutation |
| `-` | reset / decay / phrase boundary |

---

## Voice Splitting

The pipe character `|` should split a bloop into multiple voices.

Example:

```text
murmur|little-bells|soft-static
```

This becomes three independently seeded cellular voices mixed together.

Rules:

- split on `|`
- trim empty voices
- hash each voice with the full bloop seed plus the voice index
- mix voices with automatic gain compensation

Example:

```ts
const voices = source.split('|').filter(Boolean).map((part, index) => {
  return new BloopVoice(part, hashBloop(`${source}|${index}`));
});
```

---

## Cellular Automata Rule Selection

Start with elementary cellular automata rules from 0 to 255.

Derive the base rule from the hash:

```ts
const rule = hash & 0xff;
```

Then let characters bias the rule.

Examples:

```ts
if (source.includes('x')) rule ^= 0b01011010;
if (source.includes('l')) rule &= 0b11101110;
if (source.includes('z')) rule |= 0b00100100;
```

Useful rules to consider for musically interesting behavior:

- Rule 30: chaotic/noisy
- Rule 90: XOR fractal/Sierpinski
- Rule 110: complex computational behavior
- Rule 184: traffic-like motion

Do not hard-code only these rules. Let the bloop hash choose the rule, with character biasing.

---

## Audio Readout Strategy

Each audio sample should be derived from the current automata row.

Simple initial approach:

1. Select a moving window from the row.
2. Convert a small group of bits into an integer.
3. Normalize to `[-1, 1]`.
4. Apply character-derived transforms.
5. Apply simple filtering and gain safety.

Example:

```ts
function rowBitsToSample(row: Uint8Array, offset: number): number {
  let value = 0;
  for (let i = 0; i < 8; i++) {
    value = (value << 1) | row[(offset + i) % row.length];
  }
  return (value / 127.5) - 1;
}
```

Advance the CA row every N samples, where N is derived from `rate` and source characteristics.

```ts
const samplesPerGeneration = Math.max(1, Math.floor(sampleRate / generationRate));
```

Suggested default generation rate:

```ts
const generationRate = 8000;
```

This creates a bytebeat-like digital audio feel while still being based on CA evolution.

---

## Sample Rate and Rate Control

Bytebeat often changes character dramatically when sample rate changes.

Expose this as a musical control rather than only using the browser's actual audio sample rate.

Recommended properties:

```ts
rate: number;          // multiplier, default 1
virtualRate: number;   // optional, e.g. 8000, 11025, 22050, 44100
```

The browser may run at 44.1kHz or 48kHz, but the VM can operate at a virtual tick rate.

Suggested modes:

```ts
tiny: 4000
lofi: 8000
mid: 11025
clear: 22050
hi: 44100
```

This can later map to Bloopnet characters or metadata.

---

## Live Typing Behavior

For the terminal example, the bloop should be playable as it is being written.

Implementation approach:

- Listen to terminal input changes.
- Debounce changes by 30–100ms.
- Call `audioElement.bloop = currentBloop`.
- Avoid restarting the whole AudioContext for every keystroke.
- Re-seed or morph the VM state when the bloop changes.

Two possible update modes:

### Hard reset mode

Every edit resets the sound organism.

Good for clarity and reproducibility.

### Morph mode

Every edit gradually crossfades from the previous VM to the new VM.

Better UX for live typing.

Recommended Phase 1: hard reset.

Recommended Phase 2: short crossfade, 20–80ms.

---

## Web Audio Notes

Browser audio usually requires a user gesture before playback.

The terminal example should include a clear Play/Enable Audio button.

Suggested flow:

1. User clicks `Enable Audio`.
2. AudioContext is created or resumed.
3. Bloop typing begins updating the VM.
4. Optional toggle: `Play while typing`.

Do not attempt to autoplay sound without user interaction.

---

## Safety and Listening Comfort

The VM must include basic audio safety:

- clamp final output to `[-1, 1]`
- apply conservative gain by default
- avoid DC offset
- add simple highpass or DC blocker
- limit output amplitude when multiple voices are mixed
- optionally fade in/out on play/stop

Example DC blocker:

```ts
y[n] = x[n] - x[n-1] + 0.995 * y[n-1]
```

---

## Suggested First VM Algorithm

Pseudo-code:

```ts
class CellularAudioVM {
  constructor(options) {
    this.sampleRate = options.sampleRate;
    this.cellCount = options.cellCount ?? 256;
    this.setBloop(options.bloop ?? '');
  }

  setBloop(source: string) {
    this.source = normalizeBloop(source);
    this.seed = hashBloop(this.source);
    this.rule = deriveRule(this.source, this.seed);
    this.params = deriveParams(this.source, this.seed);
    this.initializeRow();
    this.sampleIndex = 0;
    this.generation = 0;
  }

  nextSample(): number {
    if (this.sampleIndex % this.params.samplesPerGeneration === 0) {
      this.stepAutomaton();
    }

    const offset = (this.sampleIndex * this.params.readSpeed + this.params.phaseOffset) % this.cellCount;
    let sample = rowBitsToSample(this.row, offset);

    sample = applyBloopTransforms(sample, this.source, this.params, this.sampleIndex);
    sample = this.dcBlock(sample);
    sample = clamp(sample * this.params.gain, -1, 1);

    this.sampleIndex++;
    return sample;
  }
}
```

---

## Transformation Strategy

Implement transforms as cheap sample functions. Do not over-engineer the first version.

Example:

```ts
function applyBloopTransforms(sample, source, params, t) {
  if (params.bitcrush) sample = Math.round(sample * params.crush) / params.crush;
  if (params.fold) sample = fold(sample * params.foldAmount);
  if (params.gate) sample *= ((t >> params.gateShift) & 1) ? 1 : 0.2;
  if (params.clip) sample = Math.tanh(sample * params.drive);
  return sample;
}
```

Character counts can become parameter weights.

Example:

```ts
const xCount = countChar(source, 'x');
params.xorAmount = xCount / source.length;
```

---

## Example Bloops for Testing

Use these as demo presets:

```text
murmur
```

Soft, low, filtered, organic.

```text
buzz
```

Harsh, resonant, digital.

```text
aeiuollll
```

Smooth vowel drone with lowpass smoothing.

```text
txxxtxxxtxxx
```

Bytebeat-like harsh digital pulse.

```text
ktt-ktt-ktt
```

Percussive rhythm.

```text
rule?thirty?ghost
```

Chaotic probabilistic texture.

```text
little bells|soft static|deep hum
```

Layered voice test.

```text
bloopnet sings itself
```

Friendly phrase test.

---

## Terminal Example Integration

Assuming there is an existing Bloop terminal input, add an audio preview panel.

Example HTML:

```html
<bloop-terminal id="terminal"></bloop-terminal>

<section class="audio-preview">
  <button id="enableAudio">Enable Audio</button>
  <label>
    <input type="checkbox" id="playWhileTyping" checked />
    Play while typing
  </label>
  <bloop-audio-vm id="audio" volume="0.25" preview></bloop-audio-vm>
</section>
```

Example JS/TS:

```ts
const terminal = document.querySelector('#terminal');
const audio = document.querySelector('bloop-audio-vm');
const enableAudio = document.querySelector('#enableAudio');
const playWhileTyping = document.querySelector('#playWhileTyping');

enableAudio.addEventListener('click', async () => {
  await audio.play();
});

terminal.addEventListener('bloop-input', (event) => {
  audio.bloop = event.detail.value;
  if (playWhileTyping.checked && !audio.playing) {
    audio.play();
  }
});
```

Adapt event names to the existing terminal implementation.

---

## Accessibility / UX Requirements

- Audio must be opt-in.
- Provide visible play/stop controls.
- Provide a volume control.
- Provide a text label explaining that the bloop is being sonified.
- Respect reduced-motion-style user preferences where relevant, but audio is more about explicit consent than motion.
- Do not play sound automatically on page load.

Suggested UI labels:

```text
Enable Audio
Play Bloop
Stop
Play while typing
Volume
Sound Rate
```

---

## Testing Plan

### Unit Tests

Test:

- bloop normalization
- hash determinism
- rule derivation determinism
- same bloop produces same first N samples
- different bloops usually produce different sample streams
- unsupported characters are handled correctly
- pipe splitting creates correct voice count

### Snapshot Tests

For several example bloops, generate the first 1024 samples and hash the resulting Float32Array. Store snapshot hashes.

This catches accidental sound-engine changes.

### Browser Tests

- component registers successfully
- `bloop` attribute updates the VM
- `play()` starts audio after user gesture
- `stop()` stops audio
- terminal typing updates the bloop

---

## Development Phases

### Phase 1 — Deterministic DSP Core

Build:

- Bloop normalizer
- hash function
- cellular automata row initializer
- elementary CA stepper
- sample readout
- basic transforms
- unit tests

No Web Component yet.

### Phase 2 — Web Audio Engine

Build:

- `BloopAudioEngine`
- audio context lifecycle
- play/stop/reset
- volume gain
- safe fade in/out

Use a simple implementation first.

### Phase 3 — Web Component

Build:

- `<bloop-audio-vm>` custom element
- observed attributes
- reflected properties
- events
- minimal internal controls if desired

### Phase 4 — Terminal Demo Integration

Build:

- enable audio button
- play while typing toggle
- volume slider
- rate slider
- bloop input listener

### Phase 5 — Polish

Build:

- crossfade on bloop change
- voice splitting with `|`
- presets
- simple visualization of CA row
- optional export to WAV

---

## Optional Visualization

The component could optionally expose or render the cellular automata state.

A tiny display would be useful:

```html
<bloop-audio-vm bloop="buzz" visual></bloop-audio-vm>
```

Display options:

- current CA row as pixels
- scrolling automata history
- waveform preview
- seed/rule metadata

Do not make visualization required for the first implementation.

---

## Important Design Principle

This should feel like Bloopnet, not like a normal synthesizer.

Avoid making a large parameter-heavy synth UI.

The primary instrument should be the bloop text itself.

Good:

```text
murmur|little-bells
```

Less good:

```text
oscillator1.frequency = 220; filter.cutoff = 800;
```

The system should reward playful typing, repetition, spelling, rhythm, and symbolic texture.

---

## Open Questions for Later

Do not block Phase 1 on these.

1. Should spaces be silence, rhythmic separators, or neutral seed characters?
2. Should `?` represent probability, mutation, or branch selection?
3. Should `-` reset the automaton, decay volume, or mark phrase boundaries?
4. Should the same bloop sound identical across browsers and sample rates?
5. Should Bloopnet audio be mono only, or should stereo position derive from the bloop?
6. Should audio bloops be transmitted as plain bloops, or as a typed Bloopnet payload class?
7. Should there be a canonical reference renderer for long-term compatibility?

---

## Suggested Initial Acceptance Criteria

The first successful implementation should satisfy:

- A TypeScript `CellularAudioVM` can generate deterministic samples from a bloop string.
- The same bloop produces the same sample sequence on repeat runs.
- A `<bloop-audio-vm>` element can play a bloop in the browser.
- Updating the `bloop` attribute changes the sound.
- A demo page lets the user type a bloop and hear it update.
- The demo does not autoplay without a user gesture.
- The sound output is gain-limited and safe by default.

---

## Claude Code Starting Prompt

Use this prompt to begin implementation:

```text
You are helping implement a Bloopnet Cellular Audio VM in TypeScript.

Create a small, well-structured TypeScript module that turns a constrained Bloopnet string into deterministic audio using a one-dimensional cellular automata engine.

The allowed Bloopnet core characters are lowercase a-z, space, pipe, question mark, and hyphen. Normalize input by lowercasing and stripping unsupported characters. Cap source strings at 64 characters.

Implement the core DSP as a DOM-independent CellularAudioVM class with:

- constructor(options)
- setBloop(source: string): void
- reset(): void
- nextSample(): number
- nextBuffer(output: Float32Array): void

Use a deterministic hash, an elementary cellular automata rule, and a 256-cell binary row. The bloop should determine the initial row, rule, and sound parameters. Do not use Math.random() for sound generation.

Add unit tests showing that the same bloop produces the same first 1024 samples and that different bloops produce different sample hashes.

Then create a Web Component named <bloop-audio-vm> that exposes bloop, volume, rate, playing, play(), stop(), and reset(). It should use the Web Audio API and must not autoplay without a user gesture.

Finally, add or modify an example page so an existing Bloop terminal input can update the <bloop-audio-vm> while the user types. Include Enable Audio, Play while typing, Volume, and Rate controls.

Keep the implementation small, readable, and testable. Favor a working deterministic prototype over a complex synthesizer.
```
