use crate::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct MatchError;

impl std::fmt::Display for MatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Match Error")
    }
}

impl std::error::Error for MatchError {}

type MatchResult = Result<usize, MatchError>;

pub trait Matcher {
    fn do_match(&self, inst: Instruction) -> MatchResult;

    fn inst(self, name: &str, cycle: usize) -> BoxedMatcher
    where
        Self: Sized + 'static,
    {
        BoxedMatcher::new(when_inst(self, String::from(name), cycle))
    }

    fn specific(self, name: &str, operands: Vec<&str>, cycle: usize) -> BoxedMatcher
    where
        Self: Sized + 'static,
    {
        BoxedMatcher::new(when_specific(
            self,
            String::from(name),
            operands.into_iter().map(String::from).collect(),
            cycle,
        ))
    }
}

impl<F> Matcher for F
where
    F: Fn(Instruction) -> MatchResult,
{
    fn do_match(&self, inst: Instruction) -> MatchResult {
        self(inst)
    }
}

pub struct BoxedMatcher {
    matcher: Box<dyn Matcher>,
}

impl BoxedMatcher {
    fn new<M>(matcher: M) -> Self
    where
        M: Matcher + 'static,
    {
        BoxedMatcher {
            matcher: Box::new(matcher),
        }
    }
}

impl Matcher for BoxedMatcher {
    fn do_match(&self, inst: Instruction) -> MatchResult {
        self.matcher.do_match(inst)
    }
}

fn when_inst<M>(matcher: M, name: String, cycle: usize) -> impl Matcher
where
    M: Matcher,
{
    move |input: Instruction| {
        if input.name == name {
            return Ok(cycle);
        }
        matcher.do_match(input)
    }
}

fn when_specific<M>(matcher: M, name: String, operands: Vec<String>, cycle: usize) -> impl Matcher
where
    M: Matcher,
{
    move |input: Instruction| {
        if input.name == name && input.operands == operands {
            return Ok(cycle);
        }
        matcher.do_match(input)
    }
}

fn unit() -> impl Matcher {
    move |_input: Instruction| Ok(1)
}

pub fn make_matcher() -> impl Matcher {
    unit()
        .specific("INC", vec!["DPTR"], 2)
        .specific("ANL", vec!["addr1B", "imm1B"], 2)
        .specific("ORL", vec!["addr1B", "imm1B"], 2)
        .specific("XRL", vec!["addr1B", "imm1B"], 2)
        .specific("MOV", vec!["Rn", "addr1B"], 2)
        .specific("MOV", vec!["addr1B", "Rn"], 2)
        .specific("MOV", vec!["addr1B", "addr1B"], 2)
        .specific("MOV", vec!["addr1B", "@Ri"], 2)
        .specific("MOV", vec!["addr1B", "imm1B"], 2)
        .specific("MOV", vec!["A", "addr1B"], 2)
        .specific("MOV", vec!["@Ri", "direct"], 2)
        .specific("MOV", vec!["DPTR", "imm2B"], 2)
        .specific("MOV", vec!["bit", "C"], 2)
        .specific("ANL", vec!["C", "bit"], 2)
        .specific("ORL", vec!["C", "bit"], 2)
        .inst("MUL", 4)
        .inst("DIV", 4)
        .inst("MOVC", 2)
        .inst("MOVX", 2)
        .inst("PUSH", 2)
        .inst("POP", 2)
        .inst("JC", 2)
        .inst("JNC", 2)
        .inst("JB", 2)
        .inst("JNB", 2)
        .inst("JZ", 2)
        .inst("JNZ", 2)
        .inst("JBC", 2)
        .inst("ACALL", 2)
        .inst("LCALL", 2)
        .inst("AJMP", 2)
        .inst("JMP", 2)
        .inst("SJMP", 2)
        .inst("CJNE", 2)
        .inst("DJNZ", 2)
        .inst("RET", 2)
        .inst("RETI", 2)
}
