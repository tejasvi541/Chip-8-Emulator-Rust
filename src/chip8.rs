use std::fs;

// --- Memory map from Austin Morlan tutorial ---
// 0x000-0x1FF reserved, 0x050-0x0A0 fonts, 0x200-0xFFF ROM
pub const START_ADDRESS: usize = 0x200;
pub const FONTSET_START_ADDRESS: usize = 0x50;
pub const FONTSET_SIZE: usize = 80;
pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;

// 16 chars x 5 bytes. Each byte = 8 pixels, 1 = on.
// e.g. F = F0,80,F0,80,80 = 11110000 / 10000000 ...
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    pub registers: [u8; 16], // V0-VF, VF = flag
    pub memory: [u8; 4096],
    pub index: u16, // I, holds memory addresses
    pub pc: u16,    // program counter
    pub stack: [u16; 16],
    pub sp: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [u8; 16],
    pub video: [u32; 64 * 32], // 0x00000000 off, 0xFFFFFFFF on
    pub opcode: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        // Build RAM, copy fonts to 0x50, start pc at 0x200.
        let mut memory = [0u8; 4096];
        memory[FONTSET_START_ADDRESS..FONTSET_START_ADDRESS + FONTSET_SIZE]
            .copy_from_slice(&FONTSET);
        Self {
            registers: [0; 16],
            memory,
            index: 0,
            pc: START_ADDRESS as u16,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
            opcode: 0,
        }
    }

    pub fn load_rom(&mut self, path: &str) -> std::io::Result<()> {
        // fs::read replaces C++ ifstream + new/delete.
        let data = fs::read(path)?;
        for (i, &b) in data.iter().enumerate() {
            self.memory[START_ADDRESS + i] = b;
        }
        Ok(())
    }

    // --- Fetch, Decode, Execute ---
    // Fetch 2 bytes -> opcode, pc += 2 BEFORE execute so
    // CALL / RET / SKIP work correctly.
    // Decode with match (Rust idiom, replaces C++ fn-pointer tables).
    pub fn cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;

        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x00FF {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => {}
            },
            0x1000 => self.op_1nnn(),
            0x2000 => self.op_2nnn(),
            0x3000 => self.op_3xkk(),
            0x4000 => self.op_4xkk(),
            0x5000 => self.op_5xy0(),
            0x6000 => self.op_6xkk(),
            0x7000 => self.op_7xkk(),
            0x8000 => match self.opcode & 0x000F {
                0x0 => self.op_8xy0(),
                0x1 => self.op_8xy1(),
                0x2 => self.op_8xy2(),
                0x3 => self.op_8xy3(),
                0x4 => self.op_8xy4(),
                0x5 => self.op_8xy5(),
                0x6 => self.op_8xy6(),
                0x7 => self.op_8xy7(),
                0xE => self.op_8xye(),
                _ => {}
            },
            0x9000 => self.op_9xy0(),
            0xA000 => self.op_annn(),
            0xB000 => self.op_bnnn(),
            0xC000 => self.op_cxkk(),
            0xD000 => self.op_dxyn(),
            0xE000 => match self.opcode & 0x00FF {
                0x009E => self.op_ex9e(),
                0x00A1 => self.op_exa1(),
                _ => {}
            },
            0xF000 => match self.opcode & 0x00FF {
                0x0007 => self.op_fx07(),
                0x000A => self.op_fx0a(),
                0x0015 => self.op_fx15(),
                0x0018 => self.op_fx18(),
                0x001E => self.op_fx1e(),
                0x0029 => self.op_fx29(),
                0x0033 => self.op_fx33(),
                0x0055 => self.op_fx55(),
                0x0065 => self.op_fx65(),
                _ => {}
            },
            _ => {}
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    // --- 34 ops. vx/vy must be usize for array indexing. ---
    pub fn op_00e0(&mut self) {
        self.video.fill(0); // CLS: clear display
    }

    pub fn op_00ee(&mut self) {
        // RET: pop return address into pc (overwrites pc += 2).
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    pub fn op_1nnn(&mut self) {
        // JP addr: no stack use.
        self.pc = self.opcode & 0x0FFF;
    }

    pub fn op_2nnn(&mut self) {
        // CALL addr: push pc (already points at next instr), jump.
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    pub fn op_3xkk(&mut self) {
        // SE Vx, byte: skip next instr if equal (pc already +2).
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        if self.registers[vx] == (self.opcode & 0xFF) as u8 {
            self.pc += 2;
        }
    }

    pub fn op_4xkk(&mut self) {
        // SNE Vx, byte.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        if self.registers[vx] != (self.opcode & 0xFF) as u8 {
            self.pc += 2;
        }
    }

    pub fn op_5xy0(&mut self) {
        // SE Vx, Vy.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        if self.registers[vx] == self.registers[vy] {
            self.pc += 2;
        }
    }

    pub fn op_6xkk(&mut self) {
        // LD Vx, byte.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[vx] = (self.opcode & 0xFF) as u8;
    }

    pub fn op_7xkk(&mut self) {
        // ADD Vx, byte. wrapping = C++ u8 overflow.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[vx] = self.registers[vx].wrapping_add((self.opcode & 0xFF) as u8);
    }

    pub fn op_8xy0(&mut self) {
        // LD Vx, Vy.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[vx] = self.registers[vy];
    }

    pub fn op_8xy1(&mut self) {
        // OR Vx, Vy.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[vx] |= self.registers[vy];
    }

    pub fn op_8xy2(&mut self) {
        // AND Vx, Vy.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[vx] &= self.registers[vy];
    }

    pub fn op_8xy3(&mut self) {
        // XOR Vx, Vy.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[vx] ^= self.registers[vy];
    }

    pub fn op_8xy4(&mut self) {
        // ADD Vx, Vy with carry in VF.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let sum = self.registers[vx] as u16 + self.registers[vy] as u16;
        self.registers[0xF] = if sum > 255 { 1 } else { 0 };
        self.registers[vx] = (sum & 0xFF) as u8;
    }

    pub fn op_8xy5(&mut self) {
        // SUB Vx, Vy. VF = NOT borrow (Vx > Vy).
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[0xF] = if self.registers[vx] > self.registers[vy] {
            1
        } else {
            0
        };
        self.registers[vx] = self.registers[vx].wrapping_sub(self.registers[vy]);
    }

    pub fn op_8xy6(&mut self) {
        // SHR Vx: save LSB in VF, then /2.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[0xF] = self.registers[vx] & 0x1;
        self.registers[vx] >>= 1;
    }

    pub fn op_8xy7(&mut self) {
        // SUBN Vx, Vy: Vx = Vy - Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[0xF] = if self.registers[vy] > self.registers[vx] {
            1
        } else {
            0
        };
        self.registers[vx] = self.registers[vy].wrapping_sub(self.registers[vx]);
    }

    pub fn op_8xye(&mut self) {
        // SHL Vx: save MSB in VF, then *2.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;
        self.registers[vx] <<= 1;
    }

    pub fn op_9xy0(&mut self) {
        // SNE Vx, Vy.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        if self.registers[vx] != self.registers[vy] {
            self.pc += 2;
        }
    }

    pub fn op_annn(&mut self) {
        // LD I, addr.
        self.index = self.opcode & 0x0FFF;
    }

    pub fn op_bnnn(&mut self) {
        // JP V0, addr.
        self.pc = self.registers[0] as u16 + (self.opcode & 0x0FFF);
    }

    pub fn op_cxkk(&mut self) {
        // RND Vx, byte: random & kk. rand crate seeds from OS.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[vx] = rand::random::<u8>() & (self.opcode & 0xFF) as u8;
    }

    pub fn op_dxyn(&mut self) {
        // DRW: XOR n-byte sprite at (Vx,Vy), VF = collision, wrap.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let h = (self.opcode & 0x000F) as usize;
        let x = self.registers[vx] as usize % VIDEO_WIDTH;
        let y = self.registers[vy] as usize % VIDEO_HEIGHT;
        self.registers[0xF] = 0;
        for row in 0..h {
            let byte = self.memory[self.index as usize + row];
            for col in 0..8 {
                if byte & (0x80 >> col) != 0 {
                    let idx = (y + row) * VIDEO_WIDTH + (x + col);
                    if self.video[idx] == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }
                    self.video[idx] ^= 0xFFFFFFFF;
                }
            }
        }
    }

    pub fn op_ex9e(&mut self) {
        // SKP Vx: skip if key Vx pressed.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.registers[vx] as usize] != 0 {
            self.pc += 2;
        }
    }

    pub fn op_exa1(&mut self) {
        // SKNP Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.registers[vx] as usize] == 0 {
            self.pc += 2;
        }
    }

    pub fn op_fx07(&mut self) {
        // LD Vx, DT.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[vx] = self.delay_timer;
    }

    pub fn op_fx0a(&mut self) {
        // LD Vx, K: wait for key. pc -= 2 re-runs instr.
        // position() replaces 16x if-else chain.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        if let Some(i) = self.keypad.iter().position(|&k| k != 0) {
            self.registers[vx] = i as u8;
        } else {
            self.pc -= 2;
        }
    }

    pub fn op_fx15(&mut self) {
        // LD DT, Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.registers[vx];
    }

    pub fn op_fx18(&mut self) {
        // LD ST, Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.registers[vx];
    }

    pub fn op_fx1e(&mut self) {
        // ADD I, Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.index = self.index.wrapping_add(self.registers[vx] as u16);
    }

    pub fn op_fx29(&mut self) {
        // LD F, Vx: font addr = 0x50 + digit*5.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.index = FONTSET_START_ADDRESS as u16 + self.registers[vx] as u16 * 5;
    }

    pub fn op_fx33(&mut self) {
        // LD B, Vx: BCD hundreds / tens / ones into I,I+1,I+2.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let mut v = self.registers[vx];
        self.memory[self.index as usize + 2] = v % 10;
        v /= 10;
        self.memory[self.index as usize + 1] = v % 10;
        v /= 10;
        self.memory[self.index as usize] = v % 10;
    }

    pub fn op_fx55(&mut self) {
        // LD [I], Vx: store V0..Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        for i in 0..=vx {
            self.memory[self.index as usize + i] = self.registers[i];
        }
    }

    pub fn op_fx65(&mut self) {
        // LD Vx, [I]: read V0..Vx.
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        for i in 0..=vx {
            self.registers[i] = self.memory[self.index as usize + i];
        }
    }
}
