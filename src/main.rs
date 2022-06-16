use std::env;
use std::process;

mod smartcropper;
mod configuration;

fn main() {
    let args: Vec<String> = env::args().collect();

    let configuration = configuration::Configuration::new(&args);

    if !configuration.valid() {
        println!("Failed to parse program arguments.");
        std::process::exit(1);
    }
    
    if let Err(error) = smartcropper::run(configuration) {
        println!("Application error! {}", error);
        
        process::exit(1);
    }
}
