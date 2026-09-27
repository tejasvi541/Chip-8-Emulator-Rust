# Chip-8 Emulator in Rust

A CHIP-8 interpreter/emulator written in Rust, inspired by [Austin Morlan's Building a CHIP-8 Emulator (C++)](https://austinmorlan.com/posts/chip8_emulator/).

## Requirements

- Rust (tested with 1.96.0)
- macOS / Linux / Windows

## Run

```sh
cargo run <Scale> <DelayMs> <ROM>
```

Example:

```sh
cargo run 10 3 roms/Tetris.ch8
```

- `Scale`: integer window scale factor for the 64x32 display.
- `DelayMs`: milliseconds between CPU cycles (try 1–4; Tetris plays well at 3).
- `ROM`: path to a `.ch8` ROM file.

## Controls

CHIP-8 keypad mapped to keyboard (per the tutorial):

| Keypad | Keyboard |
| ------ | -------- |
| 1 2 3 C | 1 2 3 4 |
| 4 5 6 D | Q W E R |
| 7 8 9 E | A S D F |
| A 0 B F | Z X C V |

`Esc` quits.

## Project structure

- `src/chip8.rs` — CPU: memory, registers, fontset, all 34 opcodes, fetch/decode/execute `cycle()`.
- `src/main.rs` — platform layer and main loop (`minifb` window, input, timed cycles).

Differences from the C++ reference: opcode dispatch uses `match` instead of member-function pointer tables, and randomness comes from the `rand` crate (OS-seeded) instead of `<random>` with a clock seed.

## Test ROMs

Try an opcode test ROM, then a game:

```sh
cargo run 10 1 test_opcode.ch8
cargo run 10 3 Tetris.ch8
```

## Acknowledgement

Logic and structure follow Austin Morlan's post linked above; this repo is a Rust port of that design.
