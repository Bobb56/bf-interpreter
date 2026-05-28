use super::irbuilder::Instr;
use std::default::Default;
use std::mem;

use gccjit::ToRValue;

const INITIAL_TAPE_LEN: usize = 1000;


struct IO {
    chars_queue: Vec<u8>
}

impl IO {
    pub fn new() -> IO {
        IO {chars_queue: Vec::new()}
    }

    fn get_new_entry(&mut self) -> () {
        let mut string = String::new();
        std::io::stdin().read_line(&mut string).expect("Error while reading console");
        string.pop();
        for char in string.chars() {
            self.chars_queue.push(char as u8);
        }
    }

    fn read_char(&mut self) -> u8 {
        if self.chars_queue.is_empty() {
            self.get_new_entry();
        }
        self.chars_queue.remove(0)
    }

    pub fn read(&mut self, n: usize, buffer: &mut [u8]) -> () {
        for i in 0..n {
            buffer[i] = self.read_char();
        }
    }

    pub fn write(buf: &Vec<u8>, pointer: usize, count: i32) -> () {
        for i in 0..count {
            print!("{}", buf[(pointer as i32 + i) as usize] as char);
        }
    }
}


pub fn run(instructions: &Vec<Instr>) -> () {
    let mut tape: Vec<u8> = vec![0 ; INITIAL_TAPE_LEN];
    let mut io= IO::new();
    let mut pointer: usize = 0;
    let mut pc: i32 = 0;
    while (pc as usize) < instructions.len() {
        match instructions[pc as usize] {
            Instr::Add(value) => tape[pointer] = ((tape[pointer] as i32 + value)%256) as u8,
            Instr::Move(value) => pointer = ((pointer as i32 + value) as usize)%tape.len(),
            Instr::Write(value) => IO::write(&tape, pointer, value),
            Instr::Read(n) => io.read(n as usize, &mut tape[pointer..=pointer+n as usize]),
            Instr::CondBr(addr) => if tape[pointer] == 0 {pc = addr as i32},
            Instr::Goto(addr) => pc = addr as i32 - 1,
            Instr::UnsolvedCondBr => panic!("This code still has unsolved condbranchs!!!")
        };
        pc += 1;
    }
}


// Code taken and adapted from https://github.com/rust-lang/gccjit.rs.git
pub fn run_jit(instructions: &Vec<Instr>) -> () {
    let context = gccjit::Context::default();
    context.set_optimization_level(gccjit::OptimizationLevel::Limited);
    
    if !codegen(&instructions[..], &context) {
        panic!("unbalanced brackets");
    }

    let result = context.compile();
    let main_result = result.get_function("bf_main");
    let main : extern "C" fn() =
        if !main_result.is_null() {
            unsafe { mem::transmute(main_result) }
        }
        else {
           panic!("failed to codegen")
        };
    main();
}



// Code taken from https://github.com/rust-lang/gccjit.rs.git and adapted to our IR
fn codegen<'a, 'ctx>(ops: &[Instr], context: &'a gccjit::Context<'ctx>) -> bool {
    // first we set up the function so that it has signature () -> void.
    let void_ty = context.new_type::<()>();
    let char_ty = context.new_type::<u8>();
    let int_ty = context.new_type::<i32>();
    // before we get started - get a reference to getchar, putchar, and memset.
    let getchar = context.new_function(None,
                                       gccjit::FunctionType::Extern,
                                       char_ty,
                                       &[],
                                       "getchar",
                                       false);
    let parameter = context.new_parameter(None, char_ty, "c");
    let putchar = context.new_function(None,
                                       gccjit::FunctionType::Extern,
                                       void_ty,
                                       &[parameter],
                                       "putchar",
                                       false);
    let memory_ty = context.new_array_type(None, char_ty, INITIAL_TAPE_LEN as u64);
    // memset definition - going to cheat a little bit and not give the C definition since
    // gcc's backend doesn't have C's notion of implicit type conversions (i.e. unsigned char[] to void*)
    let char_ptr = context.new_type::<u8>().make_pointer();
    let void_param = context.new_parameter(None, char_ptr, "ptr");
    // also here - we're lying a bit and saying that int == size_t. This obviously isn't always true
    // but it's good enough for this toy program.
    let size_t_param = context.new_parameter(None, int_ty, "size");
    let int_param = context.new_parameter(None, int_ty, "num");
    let void_ptr_ty = context.new_type::<*mut ()>();
    let memset = context.new_function(None,
                                      gccjit::FunctionType::Extern,
                                      void_ptr_ty,
                                      &[void_param, int_param, size_t_param],
                                      "memset",
                                      false);

    let brainf_main = context.new_function(None, gccjit::FunctionType::Exported, void_ty, &[], "bf_main", false);
    // next, we set up the brainfuck memory array.
    let size = context.new_rvalue_from_int(int_ty, INITIAL_TAPE_LEN as i32);
    let array = brainf_main.new_local(None, memory_ty, "memory");
    let memory_ptr = brainf_main.new_local(None, int_ty, "memory_ptr");
    let mut current_block = brainf_main.new_block("entry_block");
    // now we have to zero out the giant buffer we just allocated on the stack.
    let zero_access = context.new_array_access(None, array.to_rvalue(), context.new_rvalue_zero(int_ty));
    // A function call that is done for its side effects must be sent to add_eval.
    current_block.add_eval(None, context.new_call(None, memset, &[zero_access.get_address(None), context.new_rvalue_zero(int_ty), size]));
    let mut block_stack = vec![];
    let mut blocks = 0;
    for op in ops.iter() {
        match *op {
            Instr::Add(value) => {
                // memory[ptr] += value
                let access = context.new_array_access(None, array.to_rvalue(), memory_ptr.to_rvalue());
                current_block.add_assignment_op(None, access, gccjit::BinaryOp::Plus, context.new_rvalue_from_int(char_ty, value));
            },
            Instr::Move(value) => {
                // ptr += value
                current_block.add_assignment_op(None, memory_ptr, gccjit::BinaryOp::Plus, context.new_rvalue_from_int(int_ty, value));
            },
            Instr::CondBr(_addr) => {
                // this is the opening bracket. This represents the start of two
                // new blocks. The block that is directly ahead of us (and the
                // one that will be codegen'd next) is branched to when memory[ptr]
                // is not zero. We will create the other block now but will put
                // it on the block stack.
                let cond_block = brainf_main.new_block(&*format!("block{}", blocks));
                let true_block = brainf_main.new_block(&*format!("block{}", blocks + 1));
                let false_block = brainf_main.new_block(&*format!("block{}", blocks + 2));
                blocks += 3;
                // end the current block with a jump to the conditional block.
                current_block.end_with_jump(None, cond_block);

                current_block = cond_block;

                // end the condition block with a jump to the true_block if
                // mem[ptr] != 0, false_block otherwise
                let access = context.new_array_access(None, array.to_rvalue(), memory_ptr.to_rvalue()).to_rvalue();
                let cond = context.new_comparison(None,
                                                  gccjit::ComparisonOp::NotEquals,
                                                  access,
                                                  context.new_rvalue_zero(char_ty));
                current_block.end_with_conditional(None, cond, true_block, false_block);
                // now we are going to codegen the true branch.
                current_block = true_block;
                // we push the cond block and false block onto the stack
                // so branchright knows where to jump.
                block_stack.push((cond_block, false_block));
            }
            Instr::Goto(_addr) => {
                // end the current block with a jump to the cond block on the
                // stack.
                let (cond, next_block) = match block_stack.pop() {
                    Some(t) => t,
                    None => return false
                };
                current_block.end_with_jump(None, cond);
                // the next block is next_block.
                current_block = next_block;
            }
            Instr::Read(n) => {
                for _ in 0..n {
                    let access = context.new_array_access(None, array.to_rvalue(), memory_ptr.to_rvalue());
                    let chr = context.new_call(None, getchar, &[]);
                    current_block.add_assignment(None, access, chr);
                }
            },
            Instr::Write(n) => {
                for _ in 0..n {
                    let access = context.new_array_access(None, array.to_rvalue(), memory_ptr.to_rvalue());
                    let call = context.new_call(None, putchar, &[access.to_rvalue()]);
                    current_block.add_eval(None, call);
                }
            },
            Instr::UnsolvedCondBr => panic!("Unsolved cond branch!!!")
        }
    }
    // this program is only valid if the block stack is zero.
    if block_stack.len() != 0 {
        return false;
    }
    // finish off the last block with a ret.
    current_block.end_with_void_return(None);
    true
}
