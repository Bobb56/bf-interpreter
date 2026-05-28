mod irbuilder;
mod runtime;


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

    let instructions = irbuilder::build(&filename).expect("Please enter a valid file name");

    if let Some(_) = std::env::args().find(|string| string == "--jit") {
        runtime::run_jit(&instructions);
    }
    else {
        runtime::run(&instructions);
    }
    
    // println!("{:?}", instructions);
}
