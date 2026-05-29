use crate::irbuilder::{Instr::UnsolvedCondBr, Node::OptimizedBody};
use super::irbuilder;
use super::irbuilder::{Node, Instr};


// More powerful instructions than the basic ones
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimplInstr {
    Add(i32, i32), // Add(offset, value) <=> mem[ptr + offset] += value
    AddMul(i32, i32, i32), // AddMul(offset1, value1, offset2) <=> mem[ptr + offset1] += value1 * mem[ptr + offset2]
    Set(i32, i32),
    Move(i32),
    CondBr(i32),
    Goto(i32),
    Read(i32),
    Write(i32)
}


impl std::fmt::Display for SimplInstr {
    fn fmt(&self, _f: &mut std::fmt::Formatter<>) -> Result<(), std::fmt::Error> {
        match self {
            SimplInstr::Add(offset, count) => {
                if *offset < 0 {print!("mem[ptr+{offset}]")} else {print!("mem[ptr-{}", - *offset)} ;
                Ok(if *count < 0 {print!("-={}", -*count)} else {print!("+={count}")})
            },
            SimplInstr::Move(count) => if *count > 0 {Ok(print!("ptr+={count}"))} else {Ok(print!("ptr-={}", -count))},
            SimplInstr::Write(count) => Ok(print!(".({count})")),
            SimplInstr::Read(count) => Ok(print!(",({count})")),
            SimplInstr::CondBr(addr) => Ok(print!("CondBr({addr})")),
            SimplInstr::Goto(addr) => Ok(print!("Goto({addr})")),
            SimplInstr::AddMul(offset1, value, offset2) => {
                if *offset1 < 0 {print!("mem[ptr+{offset1}]")} else {print!("mem[ptr-{}", - *offset1)} ;
                print!("+= {value} * ");
                Ok(if *offset2 < 0 {print!("mem[ptr+{offset2}]")} else {print!("mem[ptr-{}", - *offset2)})
            },
            SimplInstr::Set(offset, value) => {
                if *offset < 0 {print!("mem[ptr+{offset}]")} else {print!("mem[ptr-{}", - *offset)};
                Ok(print!("={value}"))
            }
        }
    }
}


// This function counts the global offset of a list of instructions
pub fn count_global_offset(body: &[SimplInstr]) -> i32 {
    let mut offset = 0;
    for instr in body {
        offset += match instr {
            SimplInstr::Move(value) => value,
            _ => &0
        }
    }
    offset
}

// Returns a tuple of offsets to the current pointer representing the left and right bounds
pub fn compute_bounds(body: &[Instr]) -> (i32, i32) {
    let mut left_bound = 0;
    let mut right_bound = 0;
    let mut offset = 0;
    for instr in body {
        offset += match instr {
            Instr::Move(value) => value,
            _ => &0
        };
        if offset < left_bound {
            left_bound = offset;
        }
        if offset > right_bound {
            right_bound = offset;
        }
    }
    (left_bound, right_bound)
}

pub fn is_io_block(body: &Vec<Instr>) -> bool {
    body.iter().any(|instr| irbuilder::is_io(*instr))
}


// This function takes an instr array in argument and returns a vector of simplified operations
// It replaces add instructions + move instructions by only add instuctions with offsets and a single move at the end
pub fn simplify_body(body: &Vec<Instr>) -> Node {
    let (left_bound, right_bound) = compute_bounds(&body[..]);
    let mut mem: Vec<i32> = vec![0 ; (right_bound - left_bound + 1) as usize];
    
    assert!(left_bound <= 0);
    let initial_pointer_index: usize = - left_bound as usize;
    let mut pointer: usize = initial_pointer_index;
    
    for instr in body {
        match instr {
            Instr::Add(value) => mem[pointer] += value,
            Instr::Move(offset) => pointer = (pointer as i32 + offset) as usize,
            Instr::CondBr(_) | Instr::Goto(_) => panic!("Cannot simplify body with loop inside"),
            Instr::Read(_) | Instr::Write(_) => (),
            UnsolvedCondBr => panic!("Unsolved cond branch")
        };
    }
    
    // Writing the optimized code
    let mut simpl_instr = Vec::new();
    
    for i in 0..mem.len() {
        if mem[i] != 0 {
            simpl_instr.push(SimplInstr::Add(i as i32 + left_bound, mem[i]));
        }
    }
    
    // Add the global offset of the block
    if pointer != initial_pointer_index {
        simpl_instr.push(SimplInstr::Move(pointer as i32 + left_bound))
    }
    
    OptimizedBody(simpl_instr)
}



pub fn convert_body_to_optimized_body(body: &Vec<Instr>) -> Vec<SimplInstr> {
    let mut new_body = Vec::new();
    for instr in body {
        new_body.push(
            match instr {
                Instr::Write(a) => SimplInstr::Write(*a),
                Instr::Read(a) => SimplInstr::Read(*a),
                Instr::Add(a) => SimplInstr::Add(0, *a),
                Instr::Move(a) => SimplInstr::Move(*a),
                _ => panic!("Cannot convert this block to optimized body")
            }
        )
    };
    new_body
}




pub fn simplify_all_bodies(ir: &mut Vec<Node>) {
    for index in 0..ir.len() {
        if let Node::Body(body) = &ir[index] {
            // We don't simplify IO blocks
            if is_io_block(body) {
                ir[index] = OptimizedBody(convert_body_to_optimized_body(body));
            }
            else {
                ir[index] = simplify_body(body);
            }
        }
    }
}



pub fn simplify_simple_loops(ir: &mut Vec<Node>) {
    // Go through all nodes
    for index in 0..ir.len() {
        // Check if the node is a simple loop (meaning a loop with only one body)
        let simple_loop_index = get_simple_loop_index(ir, index);
        if simple_loop_index != -1 {
            // We get the body of the loop
            if let Node::OptimizedBody(body) = &mut ir[simple_loop_index as usize] {
                // If the loop can be optimized (e.g its global offset is zero and there are no other instructions that add and move)
                if let Some(node) = simplify_simple_loop(body) {
                    ir[index] = node;
                    println!("Old loop: {:?}", ir[simple_loop_index as usize]);
                    println!("New loop: {:?}", ir[index]);
                    println!("");
                    // We no longer need the previous version of the loop
                    ir[simple_loop_index as usize] = Node::DeletedItem;
                }
            }
        }
    }
}

// Returns the index in the global nodes list of the body of the loop if it is a simple loop
pub fn get_simple_loop_index(ir: &mut Vec<Node>, index: usize) -> i32 {
    if let Node::Loop(bodies) = &ir[index] {
        if bodies.len() == 1 {
            return bodies[0] as i32;
        }
    }
    -1
}




// Simplifies a zero-offset loop into multiplications
pub fn simplify_simple_loop(simpl_instr: &mut Vec<SimplInstr>) -> Option<Node> {
    // First, we check that the global offset of the loop is zero
    if count_global_offset(&simpl_instr[..]) == 0 {
        // We will check if the loop terminates and optimize it only if it terminates
        let mut loop_terminates = false;
        let mut new_instrs = Vec::new();
        
        for instr in simpl_instr {
            if let SimplInstr::Add(offset, value) = instr {
                // If the instruction is mem[ptr] += -1 * mem[ptr], we skip it since every other instruction uses the value of mem[ptr]
                if *offset == 0 && (*value == -1 || *value == 1) {
                    // The only way to be sure the loop terminates is that the variant is being incremented or decremented by one at each loop iteration
                    loop_terminates = true;
                }
                else {
                    new_instrs.push(SimplInstr::AddMul(*offset, *value, 0));
                }
            }
            else if let SimplInstr::Move(offset) = instr {
                new_instrs.push(SimplInstr::Move(*offset));
            }
        };
        if loop_terminates {
            // We add here, at the end of the block, the reset of the cell used as a variant for the loop
            new_instrs.push(SimplInstr::Set(0, 0));
            Some(OptimizedBody(new_instrs))
        }
        else {
            None
        }
    }
    else {
        None
    }
}



pub fn level_is_empty(ir: &Vec<Node>, level: &Vec<usize>) -> bool {
    level.is_empty() || level.iter().all(|x| node_is_empty(ir, *x))
}


// This function checks if a block of instructions or a loop is empty, in order to remove it from its predecessors in the tree
pub fn node_is_empty(ir: &Vec<Node>, index: usize) -> bool {
    match &ir[index] {
        Node::Body(instrs) => instrs.is_empty(),
        Node::OptimizedBody(instrs) => instrs.is_empty(),
        Node::Loop(body) => {
            // When a loop is empty we warn the user but don't mark it as empty since an empty loop can do infinite loop
            if level_is_empty(ir, body) {
                println!("[WARNING] Potential infinite loop or useless loop detected! ({index})");
            };
            false
        },
        _ => false
    }
}

// Removes all empty nodes of a level and returns a new level
pub fn get_cleaned_level(ir: &Vec<Node>, level: &Vec<usize>) -> Vec<usize> {
    let mut new_level = Vec::new();
    for i in level {
        if !node_is_empty(ir, *i) {
            new_level.push(*i)
        }
    };
    new_level
}




// This function removes empty nodes in a Loop or a the Root
pub fn remove_empty_nodes(ir: &mut Vec<Node>, index: usize) -> () {
    // Get the level from the node
    match &ir[index] {
        Node::Root(level) => ir[index] = Node::Root(get_cleaned_level(ir, level)),
        Node::Loop(level) => ir[index] = Node::Loop(get_cleaned_level(ir, level)),
        _ => ()
    };
}



pub fn merge_two_instr_blocks(block1: &Node, block2: &Node) -> Node {
    if let (Node::OptimizedBody(instrs1), Node::OptimizedBody(instrs2)) = (block1, block2) {
        let mut new_instrs = instrs1.clone();
        for i in 0..instrs2.len() {
            new_instrs.push(instrs2[i]);
        };
        Node::OptimizedBody(new_instrs)
    }
    else {
        panic!("Please do not call merge_two_instr_blocks with something else than blocks")
    }
}

pub fn node_is_body(ir: &Vec<Node>, index: usize) -> bool {
    match ir[index] {
        Node::OptimizedBody(_) => true,
        _ => false
    }
}

// This function merges all contiguous instruction blocks of a level
pub fn merge_level(ir: &mut Vec<Node>, level: Vec<usize>) -> Vec<usize> {
    if !level.is_empty() {
        let mut new_level = vec![level[0]];
        for i in 1..level.len() {
            // We merge the last block of new_level and the next block of level if it is a body
            if node_is_body(ir, level[i]) && node_is_body(ir, new_level[new_level.len()-1]) {
                ir[new_level[new_level.len()-1]] = merge_two_instr_blocks(&ir[new_level[new_level.len()-1]], &ir[level[i]]);
                // We no longer need this block
                ir[level[i]] = Node::DeletedItem;
            }
            else {
                new_level.push(level[i]);
            }
        };
        new_level
    }
    else {
        Vec::new()
    }
}

pub fn merge_level_in_node(ir: &mut Vec<Node>, index: usize) -> Node {
    // First, we get the level
    let level = match &ir[index] {
        Node::Loop(level) => level,
        Node::Root(level) => level,
        Node::Body(instrs) => return Node::Body(instrs.clone()),
        Node::OptimizedBody(instrs) => return Node::OptimizedBody(instrs.clone()),
        Node::DeletedItem => return Node::DeletedItem
    };
    // Then we merge it
    let new_level = merge_level(ir, level.clone());
    // And we return the new node
    match &ir[index] {
        Node::Loop(_) => Node::Loop(new_level),
        Node::Root(_) => Node::Root(new_level),
        _ => panic!("In this case whe should have already return")
    }
}


// This function refactores the ir to remove empty blocks and merge contiguous blocks
pub fn merge_blocks(ir: &mut Vec<Node>) {
    for i in 0..ir.len() {
        remove_empty_nodes(ir, i);
    }

    for i in 0..ir.len() {
        ir[i] = merge_level_in_node(ir, i);
    }
}




pub fn instr_gen_aux(ir: &Vec<Node>, level: &Vec<usize>, code: &mut Vec<SimplInstr>) {
    for item in level {
        match &ir[*item] {
            Node::OptimizedBody(instrs) => {
                for instr in instrs {
                    code.push(*instr);
                };
            },
            Node::Loop(level) => {
                // First we add a CondBr that goes nowhere and we save its index
                let condbr_index = code.len();
                code.push(SimplInstr::CondBr(-1));
                // Then generate the code of the loop
                instr_gen_aux(ir, level, code);
                // Solve the CondBr
                code[condbr_index] = SimplInstr::CondBr(code.len() as i32);
                // And add a goto at the end of the loop
                code.push(SimplInstr::Goto(condbr_index as i32));
            },
            Node::Root(level) => instr_gen_aux(ir, level, code),
            Node::Body(instrs) => {
                let optimized_instrs = convert_body_to_optimized_body(instrs);
                for instr in optimized_instrs {
                    code.push(instr);
                };
            },
            Node::DeletedItem => ()
        }
    }
}

pub fn instr_gen(ir: &Vec<Node>) -> Vec<SimplInstr> {
    let mut instructions = Vec::new();
    instr_gen_aux(ir, &vec![0 as usize], &mut instructions);
    instructions
}

/*
pub fn simplify_body2(body: &Vec<SimplInstr>) -> Node {
    
}



pub fn simplify_all_bodies2(ir: &mut Vec<Node>) {
    for index in 0..ir.len() {
        if let Node::OptimizedBody(body) = &ir[index] {
            ir[index] = simplify_body2(body);
        }
    }
}*/