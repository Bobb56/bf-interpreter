use crate::irbuilder::{Instr::UnsolvedCondBr, Node::OptimizedBody};

use super::irbuilder::{Node, Instr};


// More powerful instructions than the basic ones
#[derive(Debug)]
pub enum SimplInstr {
    Add(i32, i32), // Add(offset, value) <=> mem[ptr + offset] += value
    AddMul(i32, i32, i32), // AddMul(offset1, value1, offset2) <=> mem[ptr + offset1] += value1 * mem[ptr + offset2]
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
                if *count < 0 {Ok(print!("-={}", -*count))} else {Ok(print!("+={count}"))}
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



pub fn simplify_all_bodies(ir: &mut Vec<Node>) {
    for index in 0..ir.len() {
        if let Node::Body(body) = &ir[index] {
            ir[index] = simplify_body(body);
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
        let mut new_instrs = Vec::new();
        for instr in simpl_instr {
            if let SimplInstr::Add(offset, value) = instr {
                new_instrs.push(SimplInstr::AddMul(*offset, *value, 0));
            }
            else if let SimplInstr::Move(offset) = instr {
                new_instrs.push(SimplInstr::Move(*offset));
            }
        };
        Some(OptimizedBody(new_instrs))
    }
    else {
        None
    }
}


// TODO : Do more loop simplifications, for instance to detect value moves