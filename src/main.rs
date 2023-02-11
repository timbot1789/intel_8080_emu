use std::env;

mod processor;
fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];

    let mut processor: processor::Processor = processor::make_processor();

    let result = processor.run_program(&file_path);

    println!("{}", result);
}
