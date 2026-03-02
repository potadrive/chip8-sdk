use crate::display::Display;

struct Chip8 {
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    dt: u8,
    st: u8,
    
    // RAM
    mem: [u8; 4096],
    stack: [u16; 16],

    display: Display,

    keyboard: [bool; 16],
}

impl Chip8 {
    const TARGET_FPS: u16 = 60;
    const IPS: u16 = 1200;

    const SYSFONT: [u8; 80] = [
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
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];
    const SYSFONT_START_ADDR: usize = 0x0000;

    const KEYBOARD_LAYOUT: [Key; 16] = [
        Key::X,     Key::Key1,  Key::Key2,  Key::Key3,
        Key::Q,     Key::W,     Key::E,     Key::A,
        Key::S,     Key::D,     Key::Z,     Key::C,
        Key::Key4,  Key::R,     Key::F,     Key::V,
    ];

    fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            sp: 0,
            pc: 0x200,
            mem: [0; 4096],
            stack: [0; 16],
            dt: 0,  
            st: 0,
            display: Display::new(64, 32, 0xAAAAAA, 0x0),
            keyboard: [false; 16],
        }
    }

    fn load_fontset(&mut self, fontset: &[u8]) {
        self.mem[Self::SYSFONT_START_ADDR..Self::SYSFONT_START_ADDR + Self::SYSFONT.len()].copy_from_slice(fontset);
    }

    fn load_program(&mut self, path: &str) {
        let rom: Vec<u8> = fs::read(path).unwrap_or_else(|_| {
            eprintln!("По данному пути байт код не найден!");
            process::exit(0);
        });
        self.mem[self.pc..self.pc + rom.len()].copy_from_slice(&rom);
    }

    fn update_timers(&mut self) {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);
    }

    fn do_instruction_cycle(&mut self) {
        // fetch
        let hi = self.mem[self.pc] as u16;
        let lo = self.mem[self.pc + 1] as u16;
        let opcode = hi << 8 | lo;
        self.pc += 2;

        // Decode & Execute
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => { // CLS
                    self.display.clear();
                }
                0x00EE => { // RET
                    self.sp = self.sp.saturating_sub(1); 
                    self.pc = self.stack[self.sp] as usize;
                }
                _ => ()
            }
            0x1000 => { // JP NNN
                self.pc = (opcode & 0x0FFF) as usize;
            }
            0x2000 => { // CALL NNN
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = (opcode & 0x0FFF) as usize;
            }
            0x3000 => { // SE Vx, NN
                if self.v[((opcode & 0x0F00) >> 8) as usize] == (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            0x4000 => { // SNE Vx, NN
                if self.v[((opcode & 0x0F00) >> 8) as usize] != (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            0x5000 => match opcode & 0x000F { // SE Vx, Vy
                0x0 => {
                    if self.v[((opcode & 0x0F00) >> 8) as usize] == self.v[((opcode & 0x00F0) >> 4) as usize] {
                        self.pc += 2;
                    }
                }
                _ => ()
            }
            0x6000 => { // LD Vx, NN
                self.v[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
            }
            0x7000 => { // ADD Vx, NN 
                let x = ((opcode & 0x0F00) >> 8) as usize;
                self.v[x] = self.v[x].wrapping_add((opcode & 0x00FF) as u8);
            }
            0x8000 => match opcode & 0x000F {
                0x0 => { // LD Vx, Vy
                    self.v[((opcode & 0x0F00) >> 8) as usize] = self.v[((opcode & 0x00F0) >> 4) as usize];
                }
                0x1 => { // OR Vx, Vy
                    self.v[((opcode & 0x0F00) >> 8) as usize] |= self.v[((opcode & 0x00F0) >> 4) as usize];
                    self.v[0xF] = 0;
                }
                0x2 => { // AND Vx, Vy
                    self.v[((opcode & 0x0F00) >> 8) as usize] &= self.v[((opcode & 0x00F0) >> 4) as usize];
                    self.v[0xF] = 0;
                }
                0x3 => { // XOR Vx, Vy
                    self.v[((opcode & 0x0F00) >> 8) as usize] ^= self.v[((opcode & 0x00F0) >> 4) as usize];
                    self.v[0xF] = 0;
                }
                0x4 => { // ADD Vx, Vy
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    let result = self.v[x] as u16 + self.v[y] as u16;
                    self.v[x] = result as u8;
                    self.v[0xF] = (result >> 8) as u8;
                }
                0x5 => { // SUB Vx, Vy
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    let vx = self.v[x];
                    let vy = self.v[y];

                    self.v[x] = vx.wrapping_sub(vy);
                    self.v[0xF] = if vx >= vy { 1 } else { 0 };
                }
                0x6 => { // SHR
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let vx = self.v[x];
                    self.v[x] >>= 1;
                    self.v[0xF] = vx & 0x1;

                }
                0x7 => { // SUBN Vx, Vy
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    let vx = self.v[x];
                    let vy = self.v[y];

                    self.v[x] = vy.wrapping_sub(vx);
                    self.v[0xF] = if vy >= vx { 1 } else { 0 };
                }
                0xE => { // SHL
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let vx = self.v[x];
                    self.v[x] <<= 1;
                    self.v[0xF] = vx >> 7;
                }
                _ => ()
            }
            0x9000 => match opcode & 0x000F { // SNE Vx, Vy
                0x0 => {
                    if self.v[((opcode & 0x0F00) >> 8) as usize] != self.v[((opcode & 0x00F0) >> 4) as usize] {
                        self.pc += 2;
                    }
                }
                _ => ()
            }
            0xA000 => { // LD I, addr
                self.i = (opcode & 0x0FFF) as usize;
            }
            0xB000 => { // JP V0, addr
                self.pc = (opcode & 0x0FFF + self.v[0] as u16) as usize;

            }
            0xC000 => { // RND Vx, byte
                self.v[((opcode & 0x0F00) >> 8) as usize] = rand::random_range(0..=255) & (opcode & 0x00FF) as u8;
            }
            0xD000 => { // DRW Vx, Vy, nibble
                let x = self.v[((opcode & 0x0F00) >> 8) as usize];
                let y = self.v[((opcode & 0x00F0) >> 4) as usize];
                let n = (opcode & 0x000F) as u8;

                self.v[0xF] = if self.display.draw_sprite(
                    &self.mem[self.i..self.i + n as usize], 
                    x, 
                    y, 
                    n
                ) { 1 } else { 0 }; 
            }
            0xE000 => match opcode & 0x00FF {
                0x9E => {
                    let keycode = self.v[((opcode & 0x0F00) >> 8) as usize] as usize;
                    if keycode < 16 && self.keyboard[keycode] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    let keycode = self.v[((opcode & 0x0F00) >> 8) as usize] as usize;
                    if keycode < 16 && !self.keyboard[keycode] {
                        self.pc += 2;
                    }
                }
                _ => ()
            }

            0xF000 => match opcode & 0x00FF {
                0x07 => { // LD Vx, DT
                    self.v[((opcode & 0x0F00) >> 8) as usize] = self.dt;
                }
                0x0A => {
                    if let Some(key) = self.keyboard.iter().position(|&k| k) {
                        self.v[((opcode & 0x0F00) >> 8) as usize] = key as u8;
                    } else {
                        self.pc -= 2;  // Повторяем инструкцию
                    }
                }
                0x15 => { // LD DT, Vx
                    self.dt = self.v[((opcode & 0x0F00) >> 8) as usize];
                }
                0x18 => { // LD ST, Vx
                    self.st = self.v[((opcode & 0x0F00) >> 8) as usize];
                }
                0x1E => { // ADD I, Vx
                    self.i += self.v[((opcode & 0x0F00) >> 8) as usize] as usize;
                }
                0x29 => { // LD F, Vx
                    self.i = Self::SYSFONT_START_ADDR + self.v[((opcode & 0x0F00) >> 8) as usize] as usize;
                }
                0x33 => { // LD B, Vx
                    let num = self.v[((opcode & 0x0F00) >> 8) as usize];
                    self.mem[self.i] = num / 100;
                    self.mem[self.i + 1] = (num / 10) % 10;
                    self.mem[self.i + 2] = num % 10;
                }
                0x55 => { // LD [I], Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    for i in 0..=x {
                        self.mem[self.i + i] = self.v[i]
                    }
                    self.i += x + 1;
                } 
                0x65 => { // LD Vx, [I]
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    for i in 0..=x {
                        self.v[i] = self.mem[self.i + i];
                    }
                    self.i += x + 1;
                } 
                _ => ()
                
            }
            _ => ()
        }

    }
}