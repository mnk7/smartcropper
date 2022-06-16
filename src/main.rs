use std::process;
use clap::{App, Parser};
use rustface::Model;

mod smartcropper;
mod configuration;

#[derive(Parser)]
/// The current configuration of the program, as constructed from all available command line arguments.
pub struct Configuration {
    #[clap(forbid_empty_values = true)]
    /// The path to the image or folder of images which should be cropped.
    path: String,

    #[clap(short, long)]
    /// The path where the resulting image(s) should be stored.
    /// If not specified, the current directory is assumed.
    output_path: Option<String>,

    #[clap(short, long)]
    #[clap(forbid_empty_values = true)]
    /// The path of the rustface-model that will be used to detect faces. A default one is included.
    model_path: Option<String>,

    #[clap(short, long)]
    /// The width of the cropped image. The image is always cropped to the largest possible size, so
    /// only the ratio between the width and height parameter is considered.
    width: f32,

    #[clap(short, long)]
    /// The height of the cropped image. The image is always cropped to the largest possible size, so
    /// only the ratio between the width and height parameter is considered.
    height: f32,

    #[clap(skip)]
    /// If the given paths are directories, all images in the path directory are cropped.
    /// This flag will be set by the valid() method.
    scan_directory: bool,

    #[clap(skip)]
    /// This field will contain all paths to images in the specified directory after the
    /// valid() method has been called.
    images_in_directory: Vec<String>,

    #[clap(skip)]
    /// The model used to detect faces with rustface.
    rustface_model: Option<Model>,
}

fn main() {
    let _matches = App::new("smartcropper")
        .version("0.1")
        .author("Michael Nunhofer <MichaelNunhofer@t-online.de>")
        .about("Crops images to a specified aspect ratio while keeping faces in frame.")
        .get_about();

    let mut configuration = Configuration::parse();

    if !configuration.valid() {
        println!("Failed to parse program arguments.");
        std::process::exit(1);
    }

    if let Err(error) = smartcropper::run(configuration) {
        println!("Application error! {}", error);
        
        process::exit(1);
    }
}
