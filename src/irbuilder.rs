use super::optimizer::SimplInstr;

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

// This represents a Brainfuck program
#[derive(Debug)]
pub enum Node {
    Body(Vec<Instr>), // A block of instructions
    Loop(Vec<usize>), // A node in the tree, the integers are the successors (the body of the loop)
    Root(Vec<usize>), // Root of the tree. The integers are the nodes of the successors
    OptimizedBody(Vec<SimplInstr>)
}




pub fn print_ir(ir: &Vec<Node>) {
    for i in 0..ir.len() {
        println!("{i}: {:?}", ir[i]);
    }
}




struct IRBuilder {
    items: Vec<Node>, // Contains all the items (loops and bodies of the program)
    level_stack: Vec<usize>,
    count: i32,
    last_character: Option<char>
}


impl IRBuilder {
    pub fn new() -> IRBuilder {
        IRBuilder { items: vec![Node::Root(vec![1]), Node::Body(Vec::new())], level_stack: vec![0], count: 0, last_character: None }
    }

    pub fn get_result(mut self) -> Vec<Node> {
        if let Some(last_char) = self.last_character {
            self.add_instruction(last_char, true);
        }
        //assert!(self.level_stack.is_empty());
        self.items
    }

    fn get_current_level(&self) -> usize {
        self.level_stack[self.level_stack.len()-1]
    }

    fn get_last_block_of_current_level(&self) -> usize {
        match &self.items[self.get_current_level()] {
            Node::Root(current_level) | Node::Loop(current_level) => current_level[current_level.len()-1],
            _ => panic!("Invalid level!!! A Body cannot be a current level."),
        }
    }

    fn add_to_current_level(&mut self, node: Node) -> () {
        let items_length = self.items.len();
        let curr_level = self.get_current_level();
        match &mut self.items[curr_level] {
            Node::Root(current_level) | Node::Loop(current_level) => {
                current_level.push(items_length)
            },
            _ => panic!("Invalid level!!! A Body cannot be a current level."),
        };
        self.items.push(node);
    }

    fn open_loop(&mut self) {
        // On fait une boucle dont le premier bloc est le bloc qu'on va ajouter juste après dans items
        let new_loop = Node::Loop(vec![self.items.len()]);
        self.items.push(Node::Body(Vec::new()));
        self.add_to_current_level(new_loop);
        self.level_stack.push(self.get_last_block_of_current_level());
    }

    fn close_loop(&mut self) {
        self.level_stack.pop();
    }


    fn push_instruction_at_current_level(&mut self, instr: Instr) {
        // If the last Node of the current level is a Loop, then we add a Body
        let need_to_add_new_body = match &self.items[self.get_last_block_of_current_level()] {
            Node::Body(_) | Node::Root(_) | Node::OptimizedBody(_) => false,
            Node::Loop(_) => true
        };

        if need_to_add_new_body {
            self.add_to_current_level(Node::Body(Vec::new()));
        }

        // Then we add the new instruction
        let last_block_of_current_level = self.get_last_block_of_current_level();
        match &mut self.items[last_block_of_current_level] {
            Node::Body(instrs) => instrs.push(instr),
            _ => panic!("ERROR: THE END OF THE CURRENT LEVEL IS NOT A BODY WHEREAS WE JUST ADDED ONE")
        }
    }

        

    pub fn add_instruction(&mut self, char: char, end: bool) -> () {
        if end || add_new_instruction(char, self.last_character) {
            match self.last_character.unwrap() {
                '+' => self.push_instruction_at_current_level(Instr::Add(self.count)),
                '-' => self.push_instruction_at_current_level(Instr::Add(-self.count)),
                '>' => self.push_instruction_at_current_level(Instr::Move(self.count)),
                '<' => self.push_instruction_at_current_level(Instr::Move(-self.count)),
                '.' => self.push_instruction_at_current_level(Instr::Write(self.count)),
                ',' => self.push_instruction_at_current_level(Instr::Read(self.count)),
                '[' => {
                    self.open_loop();
                },
                ']' => {
                    self.close_loop();
                },
                _ => panic!("{char} is not a brainfuck instruction!!!")
            };
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


fn preprocess(content: String) -> Vec<Node> {
    let mut ir_builder = IRBuilder::new();
    for char in content.chars() {
        if is_instruction(char) {
            ir_builder.add_instruction(char, false);
        }
    }
    ir_builder.get_result()
}

// Returns IR of a brainfuck program
pub fn build(filename: &str) -> Result<Vec<Node>, std::io::Error> {
    match std::fs::read_to_string(filename) {
        Ok(content) => {
            Ok(preprocess(content))
        },
        Err(err) => Err(err)
    }
}