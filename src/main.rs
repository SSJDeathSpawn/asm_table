pub mod instruction;
pub mod matching;
pub mod parser;
use std::{
    collections::HashMap,
    fs,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
};

use clap::{arg, Command};
use instruction::AddressingMode;
use matching::{MatchError, Matcher};
use parser::{is_valid, ParseError};
use regex::Regex;

use crate::{
    instruction::get_addr_mode_map,
    matching::make_matcher,
    parser::{get_all_inst_variants, get_regex, get_skip_list},
};

fn cli() -> Command {
    Command::new("asm2table") 
        .about("Printing the addressing mode, machine cycle and memory bytes line-by-line used in the assembly file")
        .arg(arg!(<INPUT_FILE> "The asm file to convert").value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(-o --output <OUTPUT_FILE> "The csv file to output to").value_parser(clap::value_parser!(PathBuf)))
}

fn main() {
    let matches = cli().get_matches();

    let file = matches.get_one::<PathBuf>("INPUT_FILE").unwrap();
    println!("{:?}", file);
    if !file.exists() {
        eprintln!("File doesn't exist. Please provide a valid file!");
        return;
    }

    let contents = fs::read_to_string(file).unwrap();
    let all_inst_map = get_all_inst_variants();
    let regex_map = get_regex();
    let addr_map_mode = get_addr_mode_map();
    let skip_list = get_skip_list();
    let matcher = make_matcher();
    let mut res: Vec<[String; 4]> = vec![];
    for line in contents.lines().map(String::from) {
        let mem = get_memory(&line, &all_inst_map, &regex_map, &addr_map_mode, &skip_list);
        let cycles = get_cycle(&line, &matcher, &all_inst_map, &regex_map, &skip_list);
        let modes = get_modes(&line, &all_inst_map, &regex_map, &addr_map_mode, &skip_list);

        res.push([
            if line.contains([';']) {
                line.split_once(';').unwrap().0.trim().to_string()
            } else {
                line.clone()
            },
            if modes.is_ok() {
                modes
                    .unwrap()
                    .into_iter()
                    .map(|mode| format!("{:?}", mode))
                    .collect::<Vec<String>>()
                    .join(", ")
            } else {
                "".to_string()
            },
            if mem.is_ok() {
                format!("{:?}", mem.clone().unwrap())
            } else {
                "-1".to_string()
            },
            if cycles.is_ok() {
                format!("{:?}", cycles.clone().unwrap())
            } else {
                "-1".to_string()
            },
        ]);
    }

    if let Some(csv_file) = matches.get_one::<PathBuf>("output") {
        let mut writer = csv::Writer::from_path(csv_file).expect("File could not be opened!");
        for line in res {
            let record = line.iter().map(String::as_bytes);
            writer
                .write_record(record)
                .expect("Could not write to file!");
        }
        return;
    }

    let mut max_size: (usize, usize, usize, usize) = (
        "Instruction".len(),
        "Modes".len(),
        "Cycles".len(),
        "Memory".len(),
    );

    for line in &res {
        let to_size = |tuple: &[String; 4]| {
            (
                tuple[0].len(),
                tuple[1].len(),
                tuple[2].len(),
                tuple[3].len(),
            )
        };

        let sizes = to_size(line);

        if max_size.0 < sizes.0 {
            max_size.0 = sizes.0
        }

        if max_size.1 < sizes.1 {
            max_size.1 = sizes.1
        }

        if max_size.2 < sizes.2 {
            max_size.2 = sizes.2
        }

        if max_size.3 < sizes.3 {
            max_size.3 = sizes.3
        }
    }

    println!(
        "{:^len1$}  {:^len2$}  {:^len3$}  {:^len4$}",
        "Instruction",
        "Modes",
        "Cycles",
        "Memory",
        len1 = max_size.0,
        len2 = max_size.1,
        len3 = max_size.2,
        len4 = max_size.3
    );
    for line in res {
        println!(
            "{:<len1$}: {:>len2$}, {:>len3$}, {:>len4$}",
            line[0],
            line[1],
            line[2],
            line[3],
            len1 = max_size.0,
            len2 = max_size.1,
            len3 = max_size.2,
            len4 = max_size.3
        );
    }

    print!("Press Enter to quit...\r");
    stdout().flush().expect("Flush failed");
    let _ = stdin().read(&mut [0u8]).unwrap();
}

pub fn get_modes(
    raw_line: &str,
    all_inst: &HashMap<String, Vec<Vec<String>>>,
    regex_map: &HashMap<String, Regex>,
    addr_mode_map: &HashMap<String, AddressingMode>,
    skip_list: &[Regex],
) -> Result<Vec<AddressingMode>, ParseError> {
    is_valid(raw_line, all_inst, regex_map, skip_list)?;
    let line = if raw_line.contains([':', ';']) {
        let inter1 = raw_line
            .split_once(':')
            .map(|result| result.1)
            .unwrap_or(raw_line);
        inter1
            .split_once(';')
            .map(|res| res.0)
            .unwrap_or(inter1)
            .trim()
    } else {
        raw_line.trim()
    };
    if line.is_empty() {
        return Ok(vec![]);
    }
    let (instruction, raw_operands) = if line.contains(' ') {
        line.split_once(' ').unwrap()
    } else {
        (line, "")
    };
    if skip_list.iter().any(|reg_pat| reg_pat.is_match(line)) {
        return Ok(vec![]);
    }
    let all_operands = all_inst.get(instruction).ok_or(ParseError)?;
    let operands = raw_operands.split(',').map(str::trim);
    for ops in all_operands {
        let mut op_modes = Vec::new();
        let mut is_match = true;
        for (op1, op2) in ops.iter().zip(operands.clone()) {
            if !regex_map.get(op1).unwrap().is_match(op2) {
                is_match = false;
                break;
            }
            op_modes.push(*addr_mode_map.get(op1).unwrap());
        }
        if is_match {
            return Ok(op_modes);
        }
    }
    Err(ParseError)
}

pub fn get_memory(
    raw_line: &str,
    all_inst: &HashMap<String, Vec<Vec<String>>>,
    regex_map: &HashMap<String, Regex>,
    addr_mode_map: &HashMap<String, AddressingMode>,
    skip_list: &[Regex],
) -> Result<usize, ParseError> {
    is_valid(raw_line, all_inst, regex_map, skip_list)?;
    let line = if raw_line.contains([':', ';']) {
        let inter1 = raw_line
            .split_once(':')
            .map(|result| result.1)
            .unwrap_or(raw_line);
        inter1
            .split_once(';')
            .map(|res| res.0)
            .unwrap_or(inter1)
            .trim()
    } else {
        raw_line.trim()
    };
    if line.is_empty() {
        return Ok(0);
    }
    let (instruction, raw_operands) = if line.contains(' ') {
        line.split_once(' ').unwrap()
    } else {
        (line, "")
    };
    if skip_list.iter().any(|reg_pat| reg_pat.is_match(line)) {
        return Ok(0);
    }
    let all_operands = all_inst.get(instruction).ok_or(ParseError)?;
    let operands = raw_operands.split(',').map(str::trim);
    for ops in all_operands {
        let mut op_modes = Vec::new();
        let mut is_match = true;
        for (op1, op2) in ops.iter().zip(operands.clone()) {
            if !regex_map.get(op1).unwrap().is_match(op2) {
                is_match = false;
                break;
            }
            op_modes.push(addr_mode_map.get(op1).unwrap());
        }
        if is_match {
            let res = op_modes
                .into_iter()
                .map(AddressingMode::bytes_required)
                .sum::<usize>()
                + 1;
            return Ok(res);
        }
    }
    Err(ParseError)
}

fn get_cycle<M>(
    raw_line: &str,
    matcher: &M,
    all_inst: &HashMap<String, Vec<Vec<String>>>,
    regex_map: &HashMap<String, Regex>,
    skip_list: &[Regex],
) -> Result<usize, MatchError>
where
    M: Matcher,
{
    let line = if raw_line.contains([':', ';']) {
        let inter1 = raw_line
            .split_once(':')
            .map(|result| result.1)
            .unwrap_or(raw_line);
        inter1
            .split_once(';')
            .map(|res| res.0)
            .unwrap_or(inter1)
            .trim()
    } else {
        raw_line.trim()
    };
    if line.is_empty() {
        return Ok(0);
    }
    let (instruction, raw_operands) = if line.contains(' ') {
        line.split_once(' ').unwrap()
    } else {
        (line, "")
    };
    if skip_list.iter().any(|reg_pat| reg_pat.is_match(line)) {
        return Ok(0);
    }
    let all_operands = all_inst.get(instruction).ok_or(MatchError)?;
    let operands = raw_operands.split(',').map(str::trim);
    for ops in all_operands {
        let mut op_modes = Vec::new();
        let mut is_match = true;
        for (op1, op2) in ops.iter().zip(operands.clone()) {
            if !regex_map.get(op1).unwrap().is_match(op2) {
                is_match = false;
                break;
            }
            op_modes.push(op1.to_string());
        }
        if is_match {
            let res = matcher.do_match(instruction::Instruction {
                name: instruction.to_string(),
                operands: op_modes,
            });
            return res;
        }
    }
    Err(MatchError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sjmp() {
        let all_inst_map = get_all_inst_variants();
        let regex_map = get_regex();
        let addr_map_mode = get_addr_mode_map();
        let skip_list = get_skip_list();
        let valid = is_valid("HERE: SJMP HERE", &all_inst_map, &regex_map, &skip_list).is_ok();
        assert!(valid);
        assert_eq!(
            Ok(2),
            get_memory(
                "HERE: SJMP HERE",
                &all_inst_map,
                &regex_map,
                &addr_map_mode,
                &skip_list
            )
        );
    }

    #[test]
    fn setb() {
        let all_inst_map = get_all_inst_variants();
        let regex_map = get_regex();
        let addr_map_mode = get_addr_mode_map();
        let skip_list = get_skip_list();
        let valid = is_valid("BACK: SETB TR1", &all_inst_map, &regex_map, &skip_list).is_ok();
        assert!(valid);
        assert_eq!(
            Ok(2),
            get_memory(
                "BACK: SETB TR1",
                &all_inst_map,
                &regex_map,
                &addr_map_mode,
                &skip_list
            )
        );
    }

    #[test]
    fn clr() {
        let all_inst_map = get_all_inst_variants();
        let regex_map = get_regex();
        let addr_map_mode = get_addr_mode_map();
        let skip_list = get_skip_list();
        let valid = is_valid("CLR P2.0", &all_inst_map, &regex_map, &skip_list).is_ok();
        assert!(valid);
        assert_eq!(
            Ok(2),
            get_memory(
                "CLR P2.0",
                &all_inst_map,
                &regex_map,
                &addr_map_mode,
                &skip_list
            )
        );
    }

    #[test]
    fn dptr() {
        let all_inst_map = get_all_inst_variants();
        let regex_map = get_regex();
        let addr_map_mode = get_addr_mode_map();
        let skip_list = get_skip_list();
        let valid = is_valid("MOV DPTR, #200H", &all_inst_map, &regex_map, &skip_list).is_ok();
        assert!(valid);
        assert_eq!(
            Ok(3),
            get_memory(
                "MOV DPTR, #200H",
                &all_inst_map,
                &regex_map,
                &addr_map_mode,
                &skip_list
            )
        );
    }

    #[test]
    fn label() {
        let all_inst_map = get_all_inst_variants();
        let regex_map = get_regex();
        let addr_map_mode = get_addr_mode_map();
        let skip_list = get_skip_list();
        let valid = is_valid("WAIT:", &all_inst_map, &regex_map, &skip_list).is_ok();
        assert!(valid);
        assert_eq!(
            Ok(0),
            get_memory(
                "WAIT:",
                &all_inst_map,
                &regex_map,
                &addr_map_mode,
                &skip_list
            )
        );
    }

    #[test]
    fn jnb() {
        let all_inst_map = get_all_inst_variants();
        let regex_map = get_regex();
        let addr_map_mode = get_addr_mode_map();
        let skip_list = get_skip_list();
        let valid = is_valid("WAIT: JNB TI, WAIT", &all_inst_map, &regex_map, &skip_list).is_ok();
        assert!(valid);
        assert_eq!(
            Ok(3),
            get_memory(
                "WAIT: JNB TI, WAIT",
                &all_inst_map,
                &regex_map,
                &addr_map_mode,
                &skip_list
            )
        );
    }
}
