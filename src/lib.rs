use std::f64;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

const FONT_SET: [u8; 80] = [
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

const KEY_MAP: [u8; 16] = [
    49, 50, 51, 52, 81, 87, 69, 82, 65, 83, 68, 70, 90, 88, 67, 86,
];

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {}

#[wasm_bindgen]
pub struct Chip8Component {
    canvas: CanvasRenderingContext2d,
    scale: usize,
    interpreter: Chip8Interpreter,
}

#[wasm_bindgen]
impl Chip8Component {
    #[wasm_bindgen(constructor)]
    pub fn new(context: CanvasRenderingContext2d, scale: usize, rom: &[u8]) -> Self {
        let mut interpreter = Chip8Interpreter::default();
        interpreter.load(rom);

        Self {
            canvas: context,
            scale,
            interpreter,
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        let mut interpreter = Chip8Interpreter::default();
        interpreter.load(rom);
        self.interpreter = interpreter;
    }

    pub fn tick(&mut self) {
        self.interpreter.tick();

        if self.interpreter.draw_flag {
            self.interpreter.draw_flag = false;

            let black = JsValue::from_str("#ffffff");
            let white = JsValue::from_str("#000000");

            for i in 0..(64 * 32) {
                let mut x = i % 64;
                let mut y = i / 64;

                if self.interpreter.vram[i] > 0 {
                    self.canvas.set_fill_style(&black);
                } else {
                    self.canvas.set_fill_style(&white);
                }

                x = x * self.scale;
                y = y * self.scale;
                self.canvas
                    .fill_rect(x as f64, y as f64, self.scale as f64, self.scale as f64);
            }
        }
    }

    pub fn key_down(&mut self, key: usize) {
        let key_pressed = KEY_MAP
            .iter()
            .enumerate()
            .find(|tuple| *tuple.1 == key as u8)
            .map(|tuple| tuple.0);

        if let Some(key) = key_pressed {
            self.interpreter.set_key(key);
        }
    }

    pub fn key_up(&mut self, key: usize) {
        let key_pressed = KEY_MAP
            .iter()
            .enumerate()
            .find(|tuple| *tuple.1 == key as u8)
            .map(|tuple| tuple.0);

        if let Some(key) = key_pressed {
            self.interpreter.unset_key(key);
        }
    }
}

struct Chip8Interpreter {
    mem: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    pc: u16,
    vram: [u8; 64 * 32],
    stack: [u16; 16],
    sp: u16,
    keys: [bool; 16],

    delay_timer: u8,
    sound_timer: u8,

    await_key_press: bool,
    await_key_press_reg: u8,

    draw_flag: bool,
}

impl Default for Chip8Interpreter {
    fn default() -> Self {
        let mut ram = [0; 4096];
        for i in 0..80 {
            ram[i] = FONT_SET[i];
        }

        Self {
            mem: ram,
            registers: [0; 16],
            index: 0,
            pc: 0x200,
            vram: [0; 64 * 32],
            stack: [0; 16],
            sp: 0,
            keys: [false; 16],

            delay_timer: 0,
            sound_timer: 0,

            await_key_press: false,
            await_key_press_reg: 0,

            draw_flag: false,
        }
    }
}

impl Chip8Interpreter {
    pub fn load(&mut self, rom: &[u8]) {
        for i in 0..rom.len() {
            self.mem[0x200 + i] = rom[i];
        }
    }

    pub fn set_key(&mut self, index: usize) {
        self.keys[index] = true;

        if self.await_key_press {
            self.registers[self.await_key_press_reg as usize] = index as u8;
            self.await_key_press = false;
        }
    }

    pub fn unset_key(&mut self, index: usize) {
        self.keys[index] = false;
    }

    fn fetch(&mut self) -> u16 {
        let hi = self.mem[self.pc as usize];
        let lo = self.mem[(self.pc + 1) as usize];
        self.pc += 2;

        ((hi as u16) << 8) | lo as u16
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => {
                    for px in self.vram.as_mut() {
                        *px = 0;
                        self.draw_flag = true;
                    }
                }

                0x000E => {
                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                }
                _ => panic!("Unhandled opcode! {opcode}"),
            },

            0x1000 => {
                self.pc = opcode & 0x0FFF;
            }

            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = opcode & 0x0FFF;
            }

            0x3000 => {
                let reg = (opcode & 0xF00) >> 8;
                let cmp = opcode as u8;

                if self.registers[reg as usize] == cmp {
                    self.pc += 2;
                }
            }

            0x4000 => {
                let reg = (opcode & 0xF00) >> 8;
                let cmp = opcode as u8;

                if self.registers[reg as usize] != cmp {
                    self.pc += 2;
                }
            }

            0x5000 => {
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                let ry = ((opcode & 0x00F0) >> 4) as usize;

                if self.registers[rx] == self.registers[ry] {
                    self.pc += 2;
                }
            }

            0x6000 => {
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                self.registers[reg] = opcode as u8;
            }

            0x7000 => {
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                self.registers[reg] += opcode as u8;
            }

            0x8000 => {
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                let ry = ((opcode & 0x00F0) >> 4) as usize;

                let vx = self.registers[rx];
                let vy = self.registers[ry];

                match opcode & 0x000F {
                    0x0 => {
                        self.registers[rx] = vy;
                    }
                    0x1 => {
                        self.registers[rx] |= vy;
                    }
                    0x2 => {
                        self.registers[rx] &= vy;
                    }
                    0x3 => {
                        self.registers[rx] ^= vy;
                    }
                    0x4 => {
                        let (result, overflown) = vx.overflowing_add(vy);
                        self.registers[rx] = result;
                        self.registers[0xF] = if overflown { 1 } else { 0 };
                    }
                    0x5 => {
                        let (result, underflown) = vx.overflowing_sub(vy);
                        self.registers[rx] = result;
                        self.registers[0xF] = if underflown { 1 } else { 0 };
                    }
                    0x6 => {
                        self.registers[0xF] = if vx & 1 == 1 { 1 } else { 0 };
                        self.registers[rx] /= 2;
                    }
                    0x7 => {
                        let (result, overflown) = vy.overflowing_add(vx);
                        self.registers[rx] = result;
                        self.registers[0xF] = if overflown { 1 } else { 0 };
                    }
                    0xE => {
                        self.registers[0xF] = if vx & 1 == 1 { 1 } else { 0 };
                        self.registers[rx] = vx.wrapping_mul(2);
                    }
                    _ => panic!("Unhandled opcode! {opcode}"),
                }
            }

            0x9000 => {
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                let ry = ((opcode & 0x00F0) >> 4) as usize;

                let vx = self.registers[rx];
                let vy = self.registers[ry];

                if vx != vy {
                    self.pc += 2;
                }
            }

            0xA000 => {
                self.index = opcode & 0x0FFF;
            }

            0xB000 => {
                self.pc = (opcode & 0x0FFF) + (self.registers[0] as u16);
            }

            0xC000 => {
                use js_sys::Math::{floor, random};

                let rx = ((opcode & 0x0F00) >> 8) as usize;
                self.registers[rx] = (floor(random() * 256f64)) as u8 & (opcode as u8);
            }

            0xD000 => {
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                let ry = ((opcode & 0x00F0) >> 4) as usize;

                let x = self.registers[rx];
                let y = self.registers[ry];
                let height = opcode & 0x000F;
                self.registers[0xF] = 0;

                let mut pixel;

                for yline in 0..height {
                    pixel = self.mem[(self.index + yline) as usize];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let index = ((x as u16 + xline as u16 + ((y as u16 + yline) * 64))
                                & 0b0111_1111_1111)
                                as usize;

                            if self.vram[index] == 1 {
                                self.registers[0xF] = 1;
                            }

                            self.vram[index] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
            }

            0xE000 => {
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                let vx = self.registers[rx];

                if vx < 16 {
                    match opcode & 0x00FF {
                        0x9E => {
                            if self.keys[vx as usize] {
                                self.pc += 2;
                            }
                        }
                        0xA1 => {
                            if !self.keys[vx as usize] {
                                self.pc += 2;
                            }
                        }
                        _ => panic!("Unhandled opcode! {opcode}"),
                    }
                }
            }

            0xF000 => {
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                let vx = self.registers[rx];

                match opcode & 0x00FF {
                    0x07 => {
                        self.registers[rx] = self.delay_timer;
                    }
                    0x0A => {
                        let key_pressed = self
                            .keys
                            .iter()
                            .enumerate()
                            .find(|tuple| *tuple.1)
                            .map(|tuple| tuple.0);

                        match key_pressed {
                            Some(index) => {
                                self.registers[rx] = index as u8;
                            }

                            None => {
                                self.await_key_press = true;
                                self.await_key_press_reg = rx as u8;
                            }
                        }
                    }
                    0x15 => {
                        self.delay_timer = vx;
                    }
                    0x18 => {
                        self.sound_timer = vx;
                    }
                    0x1E => {
                        self.index += vx as u16;
                    }
                    0x29 => {
                        self.index = (vx as u16) * 5;
                    }
                    0x33 => {
                        self.mem[self.index as usize] = vx / 100;
                        self.mem[self.index as usize + 1] = (vx / 10) % 10;
                        self.mem[self.index as usize + 2] = (vx % 100) % 10;
                    }
                    0x55 => {
                        for i in 0..=rx {
                            self.mem[(self.index + i as u16) as usize] = self.registers[i as usize];
                        }
                    }
                    0x65 => {
                        for i in 0..=rx {
                            self.registers[i as usize] = self.mem[(self.index + i as u16) as usize];
                        }
                    }

                    _ => panic!("Unhandled opcode! {opcode}"),
                }
            }

            _ => panic!("Unhandled opcode! {opcode}"),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                log("beep");
            }

            self.sound_timer -= 1;
        }
    }
}
