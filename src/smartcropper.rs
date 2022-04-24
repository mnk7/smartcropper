use std::error::Error;

pub fn run(configuration: Configuration) -> Result<(), Box<dyn Error>> {

    Ok(())
}

// The current configuration of the program, as constructed from all available command line arguments.
pub struct Configuration {
    pub path: String,
    pub width: f32,
    pub height: f32,
}

impl Configuration {
    // Parses the command line arguments into usable variables
    pub fn new(args: &[String]) -> Configuration {
        let path = args[1].clone();

        let mut width = 1.0;
        let mut height = 1.0;

        if args.len() >= 4 {
            width = match args[2].parse::<f32>() {
                Ok(width) => width,
                Err(error) => {
                    println!("could not read width - {}", error);
                    1.0
                }
            };

            height = match args[3].parse::<f32>() {
                Ok(height) => height,
                Err(error) => {
                    println!("could not read height - {}", error);
                    1.0
                }
            };
        }

        Configuration {
            path,
            width,
            height,
        }
    }
}
