use crate::Configuration;

use std::fs;

impl Configuration {
    /// Returns true, if the configuration is valid.
    pub fn valid(&mut self) -> bool {
        self.images_in_directory.clear();

        let metadata = match fs::metadata(self.path.clone()) {
            Ok(metadata) => metadata,
            Err(_e) => {
                println!("{}, {}", _e, &self.path);
                return false
            },
        };

        if metadata.is_dir() {
            self.scan_directory = true;

            let paths = match fs::read_dir(self.path.clone()) {
                Ok(paths) => paths,
                Err(_e) => {
                    println!("{}, {}", _e, &self.path);
                    return false
                },
            };

            // find all images in the directory
            for dir_entry_result in paths {
                let dir_entry = match dir_entry_result {
                    Ok(dir_entry) => dir_entry,
                    Err(_e) => continue,
                };

                let path_metadata = match dir_entry.metadata() {
                    Ok(metadata) => metadata,
                    Err(_e) => continue,
                };

                let path = match dir_entry.path().into_os_string().into_string() {
                    Ok(string) => string,
                    Err(_os_string) => continue,
                };

                if path_metadata.is_file() {
                    if Configuration::path_is_image(&path) {
                        self.images_in_directory.push(path);
                    }
                }
            }
        } else {
            if Configuration::path_is_image(&self.path.clone()) {
                self.images_in_directory.push(self.path.clone());
            }
        }

        // check the other paths:

        if self.model_path.is_some() {
            let metadata_model = match fs::metadata(self.model_path.as_ref().unwrap().clone()) {
                Ok(metadata) => metadata,
                Err(_e) => {
                    println!("{}, {}", _e, &self.model_path.as_ref().unwrap());
                    return false
                },
            };

            // the model file has to be a file
            if !metadata_model.is_file() {
                println!("model file not found.");
                return false
            }

            self.rustface_model = match rustface::load_model(self.model_path.as_ref().unwrap().as_str()) {
                Ok(model) => Some(model),
                Err(_e) => {
                    println!("could not initialize model. {}", _e);
                    return false
                }
            };
        } else {
            let default_model_bytes = include_bytes!("../rustface/model/seeta_fd_frontal_v1.0.bin") as &[u8];

            self.rustface_model = match rustface::model::read_model(default_model_bytes) {
                Ok(model) => Some(model),
                Err(_e) => {
                    println!("could not read default model. {}", _e);
                    return false
                }
            };
        }

        if self.output_path.is_some() {
            let metadata_output = match fs::metadata(self.output_path.clone().unwrap_or("".to_string())) {
                Ok(metadata) => metadata,
                Err(_e) => {
                    println!("{}, {}", _e, &self.output_path.clone().unwrap_or("".to_string()));
                    return false
                },
            };

            // if we scan a directory, the output path has to point to a directory too.
            if metadata_output.is_file() {
                println!("The output path hast to be a directory.");
                return false;
            }
        }

        return true;
    }

    /// Checks, if the path belongs to an image file.
    fn path_is_image(path: &str) -> bool {
        if path.to_lowercase().ends_with(".jpg")
            || path.to_lowercase().ends_with(".jpeg")
            || path.to_lowercase().ends_with(".png") {
            return true;
        }

        return false;
    }
}