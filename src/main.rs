// Copyright © 2019 Cormac O'Brien.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate byteorder;

mod instruction;
mod types;

use std::fs::File;
use std::io::{BufRead, Read};
use std::ops::Deref;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

use instruction::Instruction;
use types::{Addr, RegId, Val};

const DIGITS: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];

struct Display {
    pixels: [[u8; 64]; 32],
}

impl Display {
    pub fn new() -> Display {
        Display {
            pixels: [[0; 64]; 32],
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [[0; 64]; 32];
    }

    pub fn draw(&mut self, x: Val, y: Val, sprite: &[u8]) -> bool {
        let mut collision = false;

        for (y_ofs, byte) in sprite.iter().enumerate() {
            // don't draw off the screen
            if y.0 as usize + y_ofs >= self.pixels.len() {
                break;
            }

            for (x_ofs, shift) in (0..8).rev().enumerate() {
                // don't draw off the screen
                if x.0 as usize + x_ofs >= self.pixels[0].len() {
                    break;
                }

                let bit = (byte >> shift) & 1u8;

                self.pixels[y.0 as usize + y_ofs][x.0 as usize + x_ofs] ^= bit;
                collision = collision || self.pixels[y.0 as usize][x.0 as usize] != bit;
            }
            println!("");
        }

        collision
    }

    pub fn print(&self) {
        for row in self.pixels.iter() {
            for pix in row.iter() {
                if *pix == 1 {
                    print!("#");
                } else {
                    print!(" ");
                }
            }

            println!("");
        }
    }
}

struct Pc(Addr);

impl Pc {
    pub fn new() -> Pc {
        Pc(Addr(0x200))
    }

    pub fn get(&self) -> Addr {
        self.0
    }

    pub fn increment(&mut self) {
        (self.0).0 = (self.0).0 + 2;
    }

    /// Increment PC if cond is true.
    pub fn increment_cond(&mut self, cond: bool) {
        (self.0).0 = (self.0).0 + 2 * (cond as u16);
    }

    pub fn jump(&mut self, addr: Addr) {
        (self.0).0 = addr.0 - 2; // pc will advance to correct address next cycle
    }
}

struct Reg(Val);

impl Reg {
    pub fn new() -> Reg {
        Reg(Val(0))
    }

    pub fn get(&self) -> Val {
        self.0
    }

    pub fn set(&mut self, k: Val) {
        self.0 = k
    }

    pub fn add(&mut self, k: Val) -> bool {
        let (val, carry) = self.0.overflowing_add(*k);
        self.0 = Val(val);
        return carry;
    }

    pub fn sub(&mut self, k: Val) -> bool {
        let (val, carry) = self.0.overflowing_sub(*k);
        self.0 = Val(val);
        return carry;
    }

    pub fn shr(&mut self) {
        use std::ops::Shr;
        self.0 = Val(self.0.shr(1));
    }

    pub fn shl(&mut self) {
        use std::ops::Shl;
        self.0 = Val(self.0.shl(1));
    }
}

impl Deref for Reg {
    type Target = Val;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Stack {
    stack: [Addr; 16],
    sp: usize,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack: [Addr(0); 16],
            sp: 0,
        }
    }

    pub fn push(&mut self, addr: Addr) {
        self.stack[self.sp] = addr;
        self.sp += 1;

        // TODO: handle stack overflow
    }

    pub fn pop(&mut self) -> Addr {
        let addr = self.stack[self.sp];
        self.sp -= 1;
        addr

        // TODO: handle stack underflow
    }
}

struct Chip8 {
    mem: [u8; 4096],
    register_v: [Reg; 16],
    register_i: Addr,
    delay: u8,
    sound: u8,
    pc: Pc,
    sp: u8,
    stack: Stack,
    display: Display,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            mem: [0; 4096],
            register_v: [
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
                Reg::new(),
            ],
            register_i: Addr(0),
            delay: 0,
            sound: 0,
            pc: Pc::new(),
            sp: 0,
            stack: Stack::new(),
            display: Display::new(),
        }
    }

    pub fn reg(&self, id: RegId) -> &Reg {
        &self.register_v[id.0 as usize]
    }

    pub fn reg_mut(&mut self, id: RegId) -> &mut Reg {
        &mut self.register_v[id.0 as usize]
    }

    pub fn set_carry(&mut self, carry: bool) {
        self.register_v[15].set(Val(carry as u8));
    }

    pub fn load<P>(&mut self, path: P)
    where
        P: AsRef<Path>,
    {
        let mut f = File::open(path).unwrap();
        let len = f.metadata().unwrap().len() as usize;
        let dst = &mut self.mem[0x200..0x200 + len];
        f.read_exact(dst).unwrap();
    }

    pub fn start(&mut self) {
        loop {
            let pc_val = self.pc.get().0 as usize;
            let mut slice = &self.mem[pc_val..pc_val + 2];
            let opcode = slice.read_u16::<BigEndian>().unwrap();
            println!("{:x}", opcode);
            let instr = Instruction::interpret(opcode).unwrap();
            println!("{:?}", instr);
            self.exec(instr);
            self.display.print();
        }
    }

    pub fn exec(&mut self, instruction: Instruction) {
        use Instruction::*;
        match instruction {
            Sys { addr } => unimplemented!(),
            Cls => self.display.clear(),
            Ret => {
                let addr = self.stack.pop();
                self.pc.jump(addr);
            }
            Jump { addr } => self.pc.jump(addr),
            Call { addr } => {
                self.stack.push(self.pc.get());
                self.pc.jump(addr);
            }
            SeVal { x, k } => self.pc.increment_cond(self.reg(x).get() == k),
            SneVal { x, k } => self.pc.increment_cond(self.reg(x).get() != k),
            SeReg { x, y } => self
                .pc
                .increment_cond(self.reg(x).get() == self.reg(y).get()),
            LdVal { x, k } => self.reg_mut(x).set(k),
            AddVal { x, k } => {
                let _carry = self.reg_mut(x).add(k); // TODO: does this ignore overflow?
            }
            LdReg { x, y } => {
                let y_val = self.reg(y).get();
                self.reg_mut(x).set(y_val);
            }
            Or { x, y } => {
                let (x_val, y_val) = (self.reg(x).get(), self.reg(y).get());
                self.reg_mut(x).set(Val(*x_val | *y_val));
            }
            And { x, y } => {
                let (x_val, y_val) = (self.reg(x).get(), self.reg(y).get());
                self.reg_mut(x).set(Val(*x_val & *y_val));
            }
            Xor { x, y } => {
                let (x_val, y_val) = (self.reg(x).get(), self.reg(y).get());
                self.reg_mut(x).set(Val(*x_val & *y_val));
            }
            AddReg { x, y } => {
                let y_val = self.reg(y).get();
                let carry = self.reg_mut(x).add(y_val);
                self.set_carry(carry);
            }
            Sub { x, y } => {
                let y_val = self.reg(y).get();
                let not_carry = !self.reg_mut(x).sub(y_val);
                self.set_carry(not_carry); // SUB sets carry flag if it does not underflow
            }
            Shr { x } => self.reg_mut(x).shr(),
            SubN { x, y } => {
                let x_val = self.reg(x).get();
                let not_carry = !self.reg_mut(y).sub(x_val);
                self.set_carry(not_carry);
            }
            Shl { x } => self.reg_mut(x).shl(),
            SneReg { x, y } => self
                .pc
                .increment_cond(self.reg(x).get() == self.reg(y).get()),
            LdI { addr } => self.register_i = addr,
            JpOfs { addr } => {
                let v0_val = self.reg(RegId(0)).get().0 as u16;
                let new_addr = Addr(addr.0 + v0_val);
                self.register_i = new_addr;
            }
            Rnd { x, k } => unimplemented!(),
            Drw { x, y, n } => {
                let sprite =
                    &self.mem[self.register_i.0 as usize..self.register_i.0 as usize + n as usize];
                let collision = self
                    .display
                    .draw(self.reg(x).get(), self.reg(y).get(), sprite);
                self.set_carry(collision);
            }
            Skp { x } => unimplemented!(),
            Sknp { x } => unimplemented!(),
            Dt { x } => {
                let val = Val(self.delay);
                self.reg_mut(x).set(val);
            }
            LdKey { x } => unimplemented!(),
            LdDt { x } => self.delay = self.reg(x).get().0,
            LdSt { x } => self.sound = self.reg(x).get().0,
            AddI { x } => self.register_i = Addr(self.register_i.0 + self.reg(x).get().0 as u16),
            LdDigit { x } => unimplemented!(),
            Bcd { x } => {
                let mut x_val = self.reg(x).get().0;
                let ones = x_val % 10;
                x_val /= 10;
                let tens = x_val % 10;
                x_val /= 10;
                let hundreds = x_val % 10;
                self.mem[self.register_i.0 as usize] = hundreds;
                self.mem[self.register_i.0 as usize + 1] = tens;
                self.mem[self.register_i.0 as usize + 2] = ones;
            }
            Store { x } => {
                for k in 0..x.0 {
                    let id = RegId(k);
                    self.mem[self.register_i.0 as usize + k as usize] = self.reg(id).get().0;
                }
            }
            Read { x } => {
                for k in 0..x.0 {
                    let id = RegId(k);
                    let val = self.mem[self.register_i.0 as usize + k as usize];
                    self.reg_mut(id).set(Val(val));
                }
            }
        }

        self.pc.increment();
    }
}

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load("roms/programs/Chip8 Picture.ch8");
    chip8.start();
}
