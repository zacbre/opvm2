use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, FromBytes, ToBytes)]
#[encoding(Json)]
pub enum Register {
    Ra,
    Rb,
    Rc,
    Rd,
    Re,
    Rf,
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
}
impl Register {
    pub fn encode(&self) -> u32 {
        match self {
            Self::Ra => 0,
            Self::Rb => 1,
            Self::Rc => 2,
            Self::Rd => 3,
            Self::Re => 4,
            Self::Rf => 5,
            Self::R0 => 6,
            Self::R1 => 7,
            Self::R2 => 8,
            Self::R3 => 9,
            Self::R4 => 10,
            Self::R5 => 11,
            Self::R6 => 12,
            Self::R7 => 13,
            Self::R8 => 14,
            Self::R9 => 15,
        }
    }

    pub fn decode(value: u32) -> Register {
        match value {
            0 => Self::Ra,
            1 => Self::Rb,
            2 => Self::Rc,
            3 => Self::Rd,
            4 => Self::Re,
            5 => Self::Rf,
            6 => Self::R0,
            7 => Self::R1,
            8 => Self::R2,
            9 => Self::R3,
            10 => Self::R4,
            11 => Self::R5,
            12 => Self::R6,
            13 => Self::R7,
            14 => Self::R8,
            15 => Self::R9,
            _ => panic!("Bad register format! {:X}", value),
        }
    }
}

macro_rules! flag_register {
    ($e:expr,bool) => {
        paste::item! {
            pub fn [<check_ $e >](&self) -> bool {
                self.$e
            }

            pub fn [<set_ $e >](&mut self, u: bool) {
                self.$e = u;
            }
        }
    };
    ($e:expr,$b:ty) => {
        paste::item! {
            #[allow(dead_code)]
            pub fn [<check_ $e >](&self) -> &$b {
                &self.$e
            }

            #[allow(dead_code)]
            pub fn [<set_ $e >](&mut self, u: $b) {
                self.$e = u;
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct Registers {
    pub ra: u64,
    pub rb: u64,
    pub rc: u64,
    pub rd: u64,
    pub re: u64,
    pub rf: u64,
    pub r0: u64,
    pub r1: u64,
    pub r2: u64,
    pub r3: u64,
    pub r4: u64,
    pub r5: u64,
    pub r6: u64,
    pub r7: u64,
    pub r8: u64,
    pub r9: u64,
    equals_flag: bool,
    greater_than_flag: bool,
    less_than_flag: bool,
    stack_len: u64,
    call_stack_len: u64,
    pc: u64,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            ra: 0,
            rb: 0,
            rc: 0,
            rd: 0,
            re: 0,
            rf: 0,
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            r9: 0,
            equals_flag: false,
            greater_than_flag: false,
            less_than_flag: false,
            stack_len: 0,
            call_stack_len: 0,
            pc: 0,
        }
    }

    pub fn get(&self, register: &Register) -> u64 {
        match register {
            Register::Ra => self.ra,
            Register::Rb => self.rb,
            Register::Rc => self.rc,
            Register::Rd => self.rd,
            Register::Re => self.re,
            Register::Rf => self.rf,
            Register::R0 => self.r0,
            Register::R1 => self.r1,
            Register::R2 => self.r2,
            Register::R3 => self.r3,
            Register::R4 => self.r4,
            Register::R5 => self.r5,
            Register::R6 => self.r6,
            Register::R7 => self.r7,
            Register::R8 => self.r8,
            Register::R9 => self.r9,
        }
    }

    pub fn set(&mut self, register: &Register, value: u64) {
        match register {
            Register::Ra => self.ra = value,
            Register::Rb => self.rb = value,
            Register::Rc => self.rc = value,
            Register::Rd => self.rd = value,
            Register::Re => self.re = value,
            Register::Rf => self.rf = value,
            Register::R0 => self.r0 = value,
            Register::R1 => self.r1 = value,
            Register::R2 => self.r2 = value,
            Register::R3 => self.r3 = value,
            Register::R4 => self.r4 = value,
            Register::R5 => self.r5 = value,
            Register::R6 => self.r6 = value,
            Register::R7 => self.r7 = value,
            Register::R8 => self.r8 = value,
            Register::R9 => self.r9 = value,
        }
    }

    flag_register!(equals_flag, bool);
    flag_register!(greater_than_flag, bool);
    flag_register!(less_than_flag, bool);
    flag_register!(stack_len, u64);
    flag_register!(call_stack_len, u64);
    flag_register!(pc, u64);

    pub fn increment_pc(&mut self) {
        self.pc += 1;
    }

    pub fn reset_flags(&mut self) {
        self.equals_flag = false;
        self.greater_than_flag = false;
        self.less_than_flag = false;
    }
}

impl TryFrom<String> for Register {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ra" => Ok(Self::Ra),
            "rb" => Ok(Self::Rb),
            "rc" => Ok(Self::Rc),
            "rd" => Ok(Self::Rd),
            "re" => Ok(Self::Re),
            "rf" => Ok(Self::Rf),
            "r0" => Ok(Self::R0),
            "r1" => Ok(Self::R1),
            "r2" => Ok(Self::R2),
            "r3" => Ok(Self::R3),
            "r4" => Ok(Self::R4),
            "r5" => Ok(Self::R5),
            "r6" => Ok(Self::R6),
            "r7" => Ok(Self::R7),
            "r8" => Ok(Self::R8),
            "r9" => Ok(Self::R9),
            &_ => Err(format!("{:?} is not a valid register", value)),
        }
    }
}
