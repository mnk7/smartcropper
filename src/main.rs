use std::env;
use std::process;

mod smartcropper;

fn main() {
    let args: Vec<String> = env::args().collect();

    let configuration = smartcropper::Configuration::new(&args);

    println!("{}, {}", configuration.path, configuration.width / configuration.height);
    
    if let Err(error) = smartcropper::run(configuration) {
        println!("Application error! {}", error);
        
        process::exit(1);
    }
}
