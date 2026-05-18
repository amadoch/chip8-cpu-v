use rand::random;

const MEMORY_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const KEYS_SIZE: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Emu {
    memory: [u8; MEMORY_SIZE],
    v_reg: [u8; NUM_REGS],
    
    program_counter: u16,
    i_reg: u16,
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
    keys: [u16; KEYS_SIZE],
    delay_reg: u8,
    sound_reg: u8,

    framebuffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT]
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            memory: [0; MEMORY_SIZE],
            v_reg: [0; NUM_REGS],

            program_counter: START_ADDR,
            i_reg: 0,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            keys: [0; KEYS_SIZE],
            delay_reg: 0,
            sound_reg: 0,

            framebuffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT]
        };
        new_emu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[
            self.stack_pointer as usize
        ]
    }

    pub fn push(&mut self, value: u16) {
        self.stack_pointer += 1;
        self.stack[
            self.stack_pointer as usize
        ] = value;
    }

    pub fn reset(&mut self) {
        self.v_reg = [0; NUM_REGS];

        self.program_counter = START_ADDR;
        self.i_reg = 0;
        self.stack = [0; STACK_SIZE];
        self.stack_pointer = 0;
        self.keys = [0; KEYS_SIZE];
        self.delay_reg = 0;
        self.sound_reg = 0;

        self.framebuffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        //fetching
        let op = self.fetch();
        //Decodifica y ejecuta
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_reg > 0 {
            self.delay_reg -= 1;
        }
        if self.sound_reg > 0 {
            if self.sound_reg == 1 {
                //TODO
            }
            self.sound_reg -= 1;
        }
    }

    pub fn get_display(&mut self) -> &[bool] {
        &self.framebuffer
    }

    pub fn key_press(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed as u16;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.memory[
            self.program_counter as usize
        ] as u16;
        let lower_byte = self.memory[
            (self.program_counter + 1) as usize
        ] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.program_counter += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (
            digit1,
            digit2,
            digit3,
            digit4
        ) {
            // 0000 - NOP
            (0, 0, 0, 0) => return,
            // 00E0 - Limpiar pantalla
            (0, 0, 0xE, 0) => { self.framebuffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT] },
            // 00EE - Retornar subrutina (función)
            (0, 0, 0xE, 0xE) => {
                let returned_address = self.pop();
                self.program_counter = returned_address;
            },
            // 1NNN - Salto
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.program_counter = nnn;
            },
            // 2NNN - Llamar a subrutina
            (2, _, _, _) => {
                let nnn= op & 0xFFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            },
            // 3XNN - VX es igual a NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (0 & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.program_counter += 2;
                };
            },
            // 4XNN - Salta si VX no es igual a NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (0 & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.program_counter += 2;
                };
            },
            // 5XY0 - Salta si VX es igual VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.program_counter += 2;
                };
            },
            // 6XNN asigna NN a VX
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (0 & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            // 7XNN suma NN a VX
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (0 & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            // 8XY0 asigna VY a VX
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            // 8XY(1, 2, 3) Operadores Bitwise
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            // 8XY4 suma VY a VX
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY5
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY6
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // 8XY7
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_reg[y].overflowing_add(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XYE
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            // 9XY0
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.program_counter += 2;
                };
            },
            // ANNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            // BNNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.program_counter = (self.v_reg[0] as u16) + nnn;
            },
            // CXNN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            // DXYN
            (0xD, _, _, _) => {
                let coord_x = self.v_reg[digit2 as usize] as u16;
                let coord_y = self.v_reg[digit3 as usize] as u16;
                let num_rows = digit4;
                let mut flipped = false;

                for y_line in 0..num_rows {
                    let address = self.i_reg + y_line as u16;
                    let pixels = self.memory[address as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (coord_x + x_line) as usize % SCREEN_WIDTH;
                            let y = (coord_y + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.framebuffer[idx];
                            self.framebuffer[idx] ^= true;
                        }
                    }
                };

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                };
            },
            // EX9E
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key != 0 {
                    self.program_counter += 2;
                };
            },
            // EXA1
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key == 0 {
                    self.program_counter += 2;
                }
            },
            // FX07
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.delay_reg;
            },
            // FX0A
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] != 0 {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    };
                };

                if !pressed {
                    self.program_counter -= 2;
                }
            },
            // FX15
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.delay_reg = self.v_reg[x];
            },
            // FX18
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.sound_reg = self.v_reg[x];
            },
            // FX1E
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            // FX29
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            // FX33 - Binary Code Decimal
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0).floor() as u8;

                self.memory[self.i_reg as usize] = hundreds;
                self.memory[(self.i_reg + 1) as usize] = tens;
                self.memory[(self.i_reg + 2) as usize] = ones;
            },
            // FX55
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.memory[i + idx] = self.v_reg[idx];
                };
            },
            // FX65
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.memory[i + idx];
                }
            },
            // Código no implementado!
            (_, _, _, _) => unimplemented!("Código de Operación NO Implementado: {op:X}"),
        }
    }
}
