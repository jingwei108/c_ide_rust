macro_rules! define_opcode {
    ($( $name:ident = $value:expr ),* $(,)?) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum OpCode {
            $( $name = $value ),*
        }

        impl OpCode {
            pub fn from_u8(value: u8) -> Option<Self> {
                match value {
                    $( $value => Some(OpCode::$name), )*
                    _ => None,
                }
            }
        }
    };
}

define_opcode! {
    Nop = 0,
    PushConst = 1,
    LoadLocal = 2,
    StoreLocal = 3,
    LoadGlobal = 4,
    StoreGlobal = 5,
    GetFrameBase = 6,
    Pop = 7,
    Dup = 8,
    Swap = 9,
    LoadMem = 10,
    StoreMem = 11,
    LoadMemByte = 12,
    StoreMemByte = 13,
    Add = 14,
    Sub = 15,
    Mul = 16,
    Div = 17,
    Mod = 18,
    Neg = 19,
    Eq = 20,
    Ne = 21,
    Lt = 22,
    Le = 23,
    Gt = 24,
    Ge = 25,
    And = 26,
    Or = 27,
    Not = 28,
    Jump = 29,
    JumpIfZero = 30,
    JumpIfNotZero = 31,
    Call = 32,
    CallHost = 33,
    Ret = 34,
    RetVoid = 35,
    StepEvent = 36,
    TrapBounds = 37,
    BitAnd = 38,
    BitOr = 39,
    BitXor = 40,
    BitNot = 41,
    Shl = 42,
    Shr = 43,
    PushConstF = 50,
    AddF = 51,
    SubF = 52,
    MulF = 53,
    DivF = 54,
    NegF = 55,
    CastI2F = 56,
    CastF2I = 57,
    EqF = 58,
    NeF = 59,
    LtF = 60,
    LeF = 61,
    GtF = 62,
    GeF = 63,
}
