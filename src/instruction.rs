#[derive(Debug)]
pub enum Instruction {
    ADDix { x: usize },
    ADDxkk { x: usize, kk: u8 },
    ADDxy { x: usize, y: usize },
    AND { x: usize, y: usize },
    CALL { nnn: u16 },
    CLS,
    DRW { x: usize, y: usize, n: u8 },
    INVALID { opcode: u16 },
    JPnnn { nnn: u16 },
    JPnnnv { nnn: u16 },
    LDbx { x: usize },
    LDfx { x: usize },
    LDix { x: usize },
    LDnnn { nnn: u16 },
    LDsx { x: usize },
    LDtx { x: usize },
    LDx { x: usize },
    LDxi { x: usize },
    LDxkk { x: usize, kk: u8 },
    LDxt { x: usize },
    LDxy { x: usize, y: usize },
    OR { x: usize, y: usize },
    RET,
    RND { x: usize, kk: u8 },
    SExkk { x: usize, kk: u8 },
    SExy { x: usize, y: usize },
    SHL { x: usize, y: usize },
    SHR { x: usize, y: usize },
    SKNP { x: usize },
    SKP { x: usize },
    SNExkk { x: usize, kk: u8 },
    SNExy { x: usize, y: usize },
    SUB { x: usize, y: usize },
    SUBN { x: usize, y: usize },
    XOR { x: usize, y: usize },
}

impl Instruction {
    pub fn from_opcode(opcode: u16) -> Self {
        use self::Instruction::*;

        let op1 = (opcode & 0xF000) >> 12;
        let op2 = (opcode & 0x0F00) >> 8;
        let op3 = (opcode & 0x00F0) >> 4;
        let op4 = opcode & 0x000F;

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        match (op1, op2, op3, op4) {
            (0x0, 0x0, 0xE, 0x0) => CLS,
            (0x0, 0x0, 0xE, 0xE) => RET,
            (0x1, _, _, _) => JPnnn { nnn },
            (0x2, _, _, _) => CALL { nnn },
            (0x3, _, _, _) => SExkk { x, kk },
            (0x4, _, _, _) => SNExkk { x, kk },
            (0x5, _, _, 0x0) => SExy { x, y },
            (0x6, _, _, _) => LDxkk { x, kk },
            (0x7, _, _, _) => ADDxkk { x, kk },
            (0x8, _, _, 0x0) => LDxy { x, y },
            (0x8, _, _, 0x1) => OR { x, y },
            (0x8, _, _, 0x2) => AND { x, y },
            (0x8, _, _, 0x3) => XOR { x, y },
            (0x8, _, _, 0x4) => ADDxy { x, y },
            (0x8, _, _, 0x5) => SUB { x, y },
            (0x8, _, _, 0x6) => SHR { x, y },
            (0x8, _, _, 0x7) => SUBN { x, y },
            (0x8, _, _, 0xE) => SHL { x, y },
            (0x9, _, _, _) => SNExy { x, y },
            (0xA, _, _, _) => LDnnn { nnn },
            (0xB, _, _, _) => JPnnnv { nnn },
            (0xC, _, _, _) => RND { x, kk },
            (0xD, _, _, _) => DRW { x, y, n },
            (0xE, _, 0x9, 0xE) => SKP { x },
            (0xE, _, 0xA, 0x1) => SKNP { x },
            (0xF, _, 0x0, 0x7) => LDxt { x },
            (0xF, _, 0x0, 0xA) => LDx { x },
            (0xF, _, 0x1, 0x5) => LDtx { x },
            (0xF, _, 0x1, 0x8) => LDsx { x },
            (0xF, _, 0x1, 0xE) => ADDix { x },
            (0xF, _, 0x2, 0x9) => LDfx { x },
            (0xF, _, 0x3, 0x3) => LDbx { x },
            (0xF, _, 0x5, 0x5) => LDix { x },
            (0xF, _, 0x6, 0x5) => LDxi { x },
            (_, _, _, _) => INVALID { opcode },
        }
    }
}
