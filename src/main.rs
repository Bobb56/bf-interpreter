use crate::irbuilder::print_ir;

mod irbuilder;
mod runtime;
mod optimizer;


pub fn get_file_name() -> String {
    let mut file = String::new();
    println!("Please enter a file name: ");

    std::io::stdin().read_line(&mut file).expect("Cannot read line");
    file.pop(); // Remove the last character (\n)
    file
}




fn main() {
    let filename = if std::env::args().len() > 1 {
        std::env::args().nth(1).unwrap()
    } else {
        get_file_name()
    };

    let mut instructions = irbuilder::build(&filename).expect("Please enter a valid file name");

    optimizer::simplify_all_bodies(&mut instructions);
    optimizer::simplify_simple_loops(&mut instructions);
    optimizer::merge_blocks(&mut instructions);

    let optimized_instructions = optimizer::instr_gen(&instructions);
    println!("{:?}", optimized_instructions);

    runtime::run_optimized(&optimized_instructions);


    /*if let Some(_) = std::env::args().find(|string| string == "--jit") {
        runtime::run_jit(&instructions);
    }
    else {
        runtime::run(&instructions);
    }*/

    
    
    //let simpl_instr = abstract_interpreter::simplify_body(&instructions);

    //println!("{:?}", simpl_instr);
}
