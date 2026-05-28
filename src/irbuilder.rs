#[derive(Debug)]
pub enum Instr {
    Add(i32),
    Move(i32),
    Write(i32),
    Read(i32),
    UnsolvedCondBr,
    CondBr(usize),
    Goto(usize)
}


struct IRBuilder {
    instructions: Vec<Instr>,
    count: i32,
    last_character: Option<char>,
    brackets_stack: Vec<usize>
}

impl IRBuilder {
    pub fn new() -> IRBuilder {
        IRBuilder { instructions: Vec::new(), brackets_stack: Vec::new(), last_character: None, count: 0}
    }

    pub fn get_result(mut self) -> Vec<Instr> {
        if let Some(last_char) = self.last_character {
            self.add_instruction(last_char, true);
        }
        assert!(self.brackets_stack.is_empty());
        self.instructions
    }

    pub fn add_instruction(&mut self, char: char, end: bool) -> () {
        if end || add_new_instruction(char, self.last_character) {
            let new_instruction = 
                match self.last_character.unwrap() {
                    '+' => Instr::Add(self.count),
                    '-' => Instr::Add(-self.count),
                    '>' => Instr::Move(self.count),
                    '<' => Instr::Move(-self.count),
                    '.' => Instr::Write(self.count),
                    ',' => Instr::Read(self.count),
                    '[' => {
                        self.brackets_stack.push(self.instructions.len());
                        Instr::UnsolvedCondBr
                    },
                    ']' => {
                        let closing_index = self.instructions.len();
                        let opening_index = self.brackets_stack.pop().expect("Syntax error : more closing brackets than opening ones");
                        self.instructions[opening_index] = Instr::CondBr(closing_index);
                        Instr::Goto(opening_index)
                    },
                    _ => panic!("{char} is not a brainfuck instruction!!!")
                };
                self.instructions.push(new_instruction);
                self.count = 1
            }
            else {
                self.count += 1
            }
            self.last_character = Some(char);
    }
}




impl std::fmt::Display for Instr {
    fn fmt(&self, _f: &mut std::fmt::Formatter<>) -> Result<(), std::fmt::Error> {
        match self {
            Instr::Add(count) => if *count > 0 {Ok(print!("+({count})"))} else {Ok(print!("-({})", -count))},
            Instr::Move(count) => if *count > 0 {Ok(print!(">({count})"))} else {Ok(print!("<({})", -count))},
            Instr::Write(count) => Ok(print!(".({count})")),
            Instr::Read(count) => Ok(print!(",({count})")),
            Instr::CondBr(addr) => Ok(print!("CondBr({addr})")),
            Instr::Goto(addr) => Ok(print!("Goto({addr})")),
            Instr::UnsolvedCondBr => Ok(print!("UnsolvedCondBr"))
        }
    }
}


fn is_instruction(char: char) -> bool {
    "+-><.,[]".contains(char)
}

fn stackable(char: char) -> bool {
    char != '[' && char != ']'
}

fn add_new_instruction(char: char, prev_char: Option<char>) -> bool {
    match prev_char {
        Some(other_char) => !stackable(other_char) || char != other_char,
        None => false
    }
}


fn preprocess(content: String) -> Vec<Instr> {
    let mut ir_builder = IRBuilder::new();
    for char in content.chars() {
        if is_instruction(char) {
            ir_builder.add_instruction(char, false);
        }
    }
    ir_builder.get_result()
}

// Returns IR of a brainfuck program
pub fn build(filename: &str) -> Result<Vec<Instr>, std::io::Error> {
    match std::fs::read_to_string(filename) {
        Ok(content) => {
            Ok(preprocess(content))
        },
        Err(err) => Err(err)
    }
}