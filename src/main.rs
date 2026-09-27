mod chip8;

use chip8::{Chip8, VIDEO_HEIGHT, VIDEO_WIDTH};
use minifb::{Key, Window, WindowOptions};
use std::{
    env,
    time::{Duration, Instant},
};

fn main() {
    // Usage: cargo run <Scale> <DelayMs> <ROM>
    // e.g. cargo run 10 3 Tetris.ch8
    // Scale enlarges 64x32 buffer for modern monitors.
    // DelayMs sets ms between cycles (tutorial had no fixed clock).
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <Scale> <DelayMs> <ROM>", args[0]);
        std::process::exit(1);
    }
    let scale: usize = args[1].parse().expect("Scale must be an integer");
    let delay = Duration::from_millis(args[2].parse::<u64>().expect("Delay must be an integer"));

    let mut chip = Chip8::new();
    chip.load_rom(&args[3]).expect("ROM read failed");

    // minifb replaces C++ Platform class (SDL window/renderer/texture).
    let mut win = Window::new(
        "CHIP-8 Emulator",
        VIDEO_WIDTH * scale,
        VIDEO_HEIGHT * scale,
        WindowOptions::default(),
    )
    .unwrap();

    // Tutorial keypad -> keyboard mapping:
    // 1 2 3 C -> 1 2 3 4 | 4 5 6 D -> Q W E R
    // 7 8 9 E -> A S D F | A 0 B F -> Z X C V
    let keymap: [(Key, usize); 16] = [
        (Key::X, 0),
        (Key::Key1, 1),
        (Key::Key2, 2),
        (Key::Key3, 3),
        (Key::Q, 4),
        (Key::W, 5),
        (Key::E, 6),
        (Key::A, 7),
        (Key::S, 8),
        (Key::D, 9),
        (Key::Z, 0xA),
        (Key::C, 0xB),
        (Key::Key4, 0xC),
        (Key::R, 0xD),
        (Key::F, 0xE),
        (Key::V, 0xF),
    ];

    // u32 buffer handed to minifb each frame (chip.video is the source).
    let mut buf = vec![0u32; VIDEO_WIDTH * VIDEO_HEIGHT];
    let mut last = Instant::now();

    // Main loop: ProcessInput + timed Cycle + Update.
    while win.is_open() && !win.is_key_down(Key::Escape) {
        chip.keypad.fill(0);
        for (k, i) in keymap {
            if win.is_key_down(k) {
                chip.keypad[i] = 1;
            }
        }

        if last.elapsed() > delay {
            last = Instant::now();
            chip.cycle();
            for (i, &px) in chip.video.iter().enumerate() {
                buf[i] = px;
            }
            win.update_with_buffer(&buf, VIDEO_WIDTH, VIDEO_HEIGHT)
                .unwrap();
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
