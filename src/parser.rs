use std::collections::HashMap;

use regex::Regex;

const INSTRUCTIONS: &str = r#"
{
	"NOP": [],
	"AJMP": [["addr1B"]],
	"RR": [["A"]],
	"INC": [["A"], ["addr1B"], ["@Ri"], ["Rn"], ["DPTR"]],
	"JBC": [["bit", "rel1B"]],
	"ACALL": [["addr1B"]],
	"LCALL": [["addr2B"]],
	"RRC": [["A"]],
	"DEC": [["A"], ["@Ri"], ["Rn"]],
	"JB": [["bit", "rel1B"]],
	"RET": [],
	"RL": [["A"]],
	"ADD": [["A", "imm1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"]],
	"JNB": [["bit", "rel1B"]],
	"RETI": [],
	"RLC": [["A"]],
	"ADDC": [["A", "imm1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"]],
	"JC": [["rel1B"]],
	"ORL": [["addr1B", "A"], ["addr1B", "imm1B"], ["A", "imm1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"], ["C", "bit"], ["bit", "C"]],
	"JNC": [["rel1B"]],
	"ANL": [["addr1B", "A"], ["addr1B", "imm1B"], ["A", "imm1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"], ["C", "bit"], ["bit", "C"]],
	"JZ": [["rel1B"]],
	"XRL": [["addr1B", "A"], ["addr1B", "imm1B"], ["A", "imm1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"]],
	"JNZ": [["rel1B"]],
	"JMP": [["@A+DPTR"]],
	"MOV": [["A", "imm1B"], ["addr1B", "imm1B"], ["@Ri", "imm1B"], ["Rn", "imm1B"], ["addr1B", "addr1B"],["addr1B", "@Ri"], ["addr1B", "Rn"], ["DPTR", "imm2B"], ["bit", "C"], ["C", "bit"], ["@Ri", "addr1B"], ["Rn", "addr1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"], ["addr1B", "A"], ["@Ri", "A"], ["Rn", "A"]],
	"SJMP": [["rel1B"]],
	"MOVC": [["A", "@A+DPTR"], ["A", "@A+PC"]],
	"DIV": [["AB"]],
	"SUBB": [["A", "imm1B"], ["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"]],
	"MUL": [["AB"]],
	"CPL": [["bit"], ["C"], ["A"]],
	"CJNE": [["A", "imm1B", "rel1B"], ["A", "addr1B", "rel1B"], ["@Ri", "imm1B", "rel1B"], ["Rn", "imm1B", "rel1B"]],
	"PUSH": [["addr1B"]],
	"CLR": [["bit"], ["C"], ["A"]],
	"SWAP": [["A"]],
	"XCH": [["A", "addr1B"], ["A", "@Ri"], ["A", "Rn"]],
	"POP": [["addr1B"]],
	"SETB": [["bit"], ["C"]],
	"DA": [["A"]],
	"DJNZ": [["addr1B", "rel1B"], ["Rn", "rel1B"]],
	"XCHD": [["A", "@Ri"]],
	"MOVX": [["A", "@DPTR"], ["A", "@Ri"], ["@DPTR", "A"], ["@Ri", "A"]],
}
"#;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error while parsing")
    }
}

pub fn get_all_inst_variants() -> HashMap<String, Vec<Vec<String>>> {
    ron::from_str(INSTRUCTIONS).expect("Error when parsing instructions.ron")
}

pub fn get_regex() -> HashMap<String, Regex> {
    let mut res = HashMap::new();
    res.insert(String::from("A"), Regex::new(r"^A$").unwrap());
    res.insert(String::from("AB"), Regex::new(r"^AB$").unwrap());
    res.insert(String::from("C"), Regex::new(r"^C$").unwrap());
    res.insert(String::from("DPTR"), Regex::new(r"^DPTR$").unwrap());
    res.insert(
        String::from("@A+DPTR"),
        Regex::new(r"^@A *\+ *DPTR$").unwrap(),
    );
    res.insert(String::from("@A+PC"), Regex::new(r"^@A *\+ *PC$").unwrap());
    res.insert(String::from("@DPTR"), Regex::new(r"^@DPTR$").unwrap());
    res.insert(String::from("@Ri"), Regex::new(r"^@R[0-1]$").unwrap());
    res.insert(String::from("Rn"), Regex::new(r"^R[0-7]$").unwrap());
    res.insert(
        String::from("addr1B"),
        Regex::new(r"^((0*([1-9][A-F0-9]|0[0-9A-F]{1,2})H)|(0*[0-1]{1,8}B)|(0*[0-9]{1,3}D?)|B|(TMOD|T(L|H)[0-1]|SCON|PCON|SBUF))$").unwrap(),
    );
    res.insert(
        String::from("imm1B"),
        Regex::new(r"^#((0*([1-9][A-F0-9]|0[0-9A-F]{1,2})H)|(0*[0-1]{1,8}B)|(-?0*[0-9]{1,3}D?))$")
            .unwrap(),
    );
    res.insert(
        String::from("addr2B"),
        Regex::new(
            r"^((0*([1-9][A-F0-9]{1,3}|0[0-9A-F]{1,4})H)|(0*[0-1]{1,16}B)|(0*[0-9]{1,5}D?)|B)$",
        )
        .unwrap(),
    );
    res.insert(
        String::from("imm2B"),
        Regex::new(
            r"^#((0*([1-9][A-F0-9]{1,3}|0[0-9A-F]{1,4})H)|(0*[0-1]{1,16}B)|(0*[0-9]{1,5}D?))$",
        )
        .unwrap(),
    );
    res.insert(
        String::from("rel1B"),
        Regex::new(r"^((0*([1-9][A-F0-9]|0[0-9A-F]{1,2})H)|(0*[0-1]{1,8}B)|(0*[0-9]{1,3}D?)|([A-Z][A-Z0-9_-]*))$").unwrap(),
    );
    res.insert(
        String::from("bit"),
        Regex::new(r"^((0*([1-9][A-F0-9]|0[0-9A-F]{1,2})H)|(0*[0-1]{1,8}B)|(0*[0-9]{1,3}D?)|((P[0-7]|ACC).[0-7])|(T(F|R)[0-1])|((T|R)I))$").unwrap(),
    );

    res
}

pub fn get_skip_list() -> Vec<Regex> {
    vec![
        Regex::new(r"^END$").unwrap(),
        Regex::new(r"^ORG.+$").unwrap(),
        Regex::new(r"^DB.+$").unwrap(),
        Regex::new(r"^.+EQU.+$").unwrap(),
    ]
}

pub fn is_valid(
    raw_line: &str,
    all_inst: &HashMap<String, Vec<Vec<String>>>,
    regex_map: &HashMap<String, Regex>,
    skip_list: &[Regex],
) -> Result<(), ParseError> {
    let line = if raw_line.contains([':', ';']) {
        let inter1 = raw_line
            .split_once(':')
            .map(|result| result.1)
            .unwrap_or(raw_line);
        inter1.split_once(';')
            .map(|res| res.0)
            .unwrap_or(inter1)
            .trim()
    } else {
        raw_line.trim()
    };
    if line.is_empty() {
        return Ok(());
    }
    let (instruction, raw_operands) = if line.contains(' ') {
        line.split_once(' ').unwrap()
    } else {
        (line, "")
    };
    if skip_list.iter().any(|reg_pat| reg_pat.is_match(line)) {
        return Ok(());
    }
    let all_operands = all_inst.get(instruction).ok_or(ParseError)?;
    if all_operands.is_empty() {
        return raw_operands
            .trim()
            .is_empty()
            .then_some(())
            .ok_or(ParseError);
    }
    let operands = raw_operands.split(',').map(str::trim);
    for ops in all_operands {
        let mut is_match = true;
        for (op1, op2) in ops.iter().zip(operands.clone()) {
            if !regex_map.get(op1).unwrap().is_match(op2) {
                is_match = false;
                break;
            }
        }
        if is_match {
            return Ok(());
        }
    }
    Err(ParseError)
}
