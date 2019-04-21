use crate::types::{Addr, RegId, Val};

#[derive(Debug)]
pub enum Instruction {
    Sys { addr: Addr },
    Cls,
    Ret,
    Jump { addr: Addr },
    Call { addr: Addr },
    SeVal { x: RegId, k: Val },
    SneVal { x: RegId, k: Val },
    SeReg { x: RegId, y: RegId },
    LdVal { x: RegId, k: Val },
    AddVal { x: RegId, k: Val },
    LdReg { x: RegId, y: RegId },
    Or { x: RegId, y: RegId },
    And { x: RegId, y: RegId },
    Xor { x: RegId, y: RegId },
    AddReg { x: RegId, y: RegId },
    Sub { x: RegId, y: RegId },
    Shr { x: RegId },
    SubN { x: RegId, y: RegId },
    Shl { x: RegId },
    SneReg { x: RegId, y: RegId },
    LdI { addr: Addr },
    JpOfs { addr: Addr },
    Rnd { x: RegId, k: Val },
    Drw { x: RegId, y: RegId, n: u8 },
    Skp { x: RegId },
    Sknp { x: RegId },
    Dt { x: RegId },
    LdKey { x: RegId },
    LdDt { x: RegId },
    LdSt { x: RegId },
    AddI { x: RegId },
    LdDigit { x: RegId },
    Bcd { x: RegId },
    Store { x: RegId },
    Read { x: RegId },
}

impl Instruction {
    pub fn interpret(opcode: u16) -> Option<Instruction> {
        use Instruction::*;

        let b = bytes(opcode);
        let n = nibbles(opcode);
        let tail = opcode & 0xFFF;
        let x = RegId(n[1]);
        let y = RegId(n[2]);

        let inst = match n[0] {
            0x0 => match tail {
                0x0E0 => Cls,                                  // 00E0 -> CLS
                0x0EE => Ret,                                  // 00EE -> RET
                _ => return None,
            }
            0x1 => Jump { addr: Addr(tail) },                  // 1nnn -> JP nnn
            0x2 => Call { addr: Addr(tail) },                  // 2nnn -> CALL nnn
            0x3 => SeVal { x, k: Val(b[1]) },     // 3xkk -> SE Vx, kk
            0x4 => SneVal { x, k: Val(b[1]) },    // 4xkk -> SNE Vx, kk
            0x5 => match n[3] {
                0 => SeReg { x, y }, // 5xy0 -> SE Vx, Vy
                _ => return None,
            }
            0x6 => LdVal { x, k: Val(b[1]) },     // 6xkk -> LD Vx, kk
            0x7 => AddVal { x, k: Val(b[1]) },    // 7xkk -> ADD Vx, kk
            0x8 => match n[3] {
                0x0 => LdReg { x, y },                         // 8xy0 -> LD Vx, Vy
                0x1 => Or { x, y },                            // 8xy1 -> OR Vx, Vy
                0x2 => And { x, y },                           // 8xy2 -> AND Vx, Vy
                0x3 => Xor { x, y },                           // 8xy3 -> XOR Vx, Vy
                0x4 => AddReg { x, y },                        // 8xy4 -> ADD Vx, Vy
                0x5 => Sub { x, y },                           // 8xy5 -> SUB Vx, Vy
                0x6 => Shr { x },                           // 8xy6 -> SHR Vx, Vy
                0x7 => SubN { x, y },                          // 8xy7 -> SUBN Vx, Vy
                0xE => Shl { x },                           // 8xyE -> SHL Vx, Vy
                _ => return None,
            }
            0x9 => match n[3] {
                0 => SneReg { x, y }, // 9xy0 -> SNE Vx, Vy
                _ => return None,
            }
            0xA => LdI { addr: Addr(tail) },
            0xB => JpOfs { addr: Addr(tail) },
            0xC => Rnd { x, k: Val(b[1]) },
            0xD => Drw { x, y, n: n[3] },
            0xE => match b[1] {
                0x9E => Skp { x },
                0xA1 => Sknp { x },
                _ => return None,
            }
            0xF => match b[1] {
                0x07 => Dt { x },
                0x0A => LdKey { x },
                0x15 => LdDt { x },
                0x18 => LdSt { x },
                0x1E => AddI { x },
                0x29 => LdDigit { x },
                0x33 => Bcd { x },
                0x55 => Store { x },
                0x65 => Read { x },
                _ => return None,
            }

            _ => unimplemented!(),
        };

        Some(inst)
    }
}

fn bytes(x: u16) -> [u8; 2] {
    let lo = (x & std::u8::MAX as u16) as u8;
    let hi = (x >> 8) as u8;

    [hi, lo]
}

fn nibbles(x: u16) -> [u8; 4] {
    let mut result = [0; 4];

    for i in 0..4 {
        result[3 - i] = ((x >> (4 * i)) & 0xF) as u8;
    }

    return result;
}

