/// The current configuration of the program, as constructed from all available command line arguments.
pub struct Configuration {
    path: String,
    output_path: String,
    model_path: String,
    width: f32,
    height: f32,
    valid: bool,
}

impl Configuration {
    // Parses the command line arguments into usable variables
    pub fn new(args: &[String]) -> Configuration {
        // weather the configuration is valid
        let mut valid = true;

        let mut path = String::from("./");
        let mut output_path = String::from("./out");
        let mut model_path = String::from("./");

        if args.len() >= 4 {
            path = args[1].clone();
            output_path = args[2].clone();
            model_path = args[3].clone();
        } else {
            valid = false
        }

        let mut width = 1.0;
        let mut height = 1.0;

        if args.len() >= 4 {
            width = match args[4].parse::<f32>() {
                Ok(width) => width,
                Err(error) => {
                    println!("could not read width - {}", error);
                    1.0
                }
            };

            height = match args[5].parse::<f32>() {
                Ok(height) => height,
                Err(error) => {
                    println!("could not read height - {}", error);
                    1.0
                }
            };
        } else {
            valid = false
        }

        Configuration {
            path,
            output_path,
            model_path,
            width,
            height,
            valid,
        }
    }

    pub fn path(&self) -> &str {
        &self.path[..]
    }

    pub fn output_path(&self) -> &str {
        &self.output_path[..]
    }

    pub fn model_path(&self) -> &str {
        &self.model_path[..]
    }

    pub fn width(&self) -> &f32 {
        &self.width
    }

    pub fn height(&self) -> &f32 {
        &self.height
    }

    pub fn valid(&self) -> &bool {
        &self.valid
    }
}