use std::{collections::HashMap, fmt::Debug};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Immediate(bool), // is 2
    Direct(bool),    // is 2
    RegisterDirect,
    RegisterIndirect,
    Indexed,
    Implied,
}

impl Debug for AddressingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(_) => f.write_str("Immediate"),
            Self::Direct(_) => f.write_str("Direct"),
            Self::RegisterDirect => f.write_str("Register"),
            Self::Indexed => f.write_str("Indexed"),
            Self::RegisterIndirect => f.write_str("Indirect"),
            Self::Implied => f.write_str("Implied")
        }
    }
}

pub struct Instruction {
    pub name: String,
    pub operands: Vec<String>
}

impl AddressingMode {
    pub fn bytes_required(&self) -> usize {
        match self {
            AddressingMode::Immediate(is_two) => 1 + (if *is_two { 1 } else { 0 }),
            AddressingMode::Indexed => 0,
            AddressingMode::Direct(is_two) => 1 + (if *is_two { 1 } else { 0 }),
            AddressingMode::Implied => 0,
            AddressingMode::RegisterDirect => 0,
            AddressingMode::RegisterIndirect => 0,
        }
    }
}

pub fn get_addr_mode_map() -> HashMap<String, AddressingMode> {
    let mut res = HashMap::new();
    res.insert(String::from("A"), AddressingMode::RegisterDirect);
    res.insert(String::from("C"), AddressingMode::RegisterDirect);
    res.insert(String::from("Rn"), AddressingMode::RegisterDirect);
    res.insert(String::from("DPTR"), AddressingMode::RegisterDirect);
    res.insert(String::from("@Ri"), AddressingMode::RegisterIndirect);
    res.insert(String::from("@DPTR"), AddressingMode::RegisterIndirect);
    res.insert(String::from("imm1B"), AddressingMode::Immediate(false));
    res.insert(String::from("imm2B"), AddressingMode::Immediate(true));
    res.insert(String::from("addr1B"), AddressingMode::Direct(false));
    res.insert(String::from("addr2B"), AddressingMode::Direct(true));
    res.insert(String::from("rel1B"), AddressingMode::Direct(false));
    res.insert(String::from("bit"), AddressingMode::Direct(false));
    res.insert(String::from("@A+DPTR"), AddressingMode::Indexed);
    res.insert(String::from("@A+PC"), AddressingMode::Indexed);
    res
}
