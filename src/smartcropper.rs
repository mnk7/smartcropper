use std::error::Error;

use anyhow::anyhow;
use anyhow::Result;

use opencv::prelude::*;

use image::RgbImage;
use ndarray::ArrayView3;

pub fn run(configuration: Configuration) -> Result<(), Box<dyn Error>> {
    // read the image from a file
    let image = opencv::imgcodecs::imread(&configuration.path, opencv::imgcodecs::IMREAD_COLOR)?;

    // Use Orb
    let mut orb = <dyn opencv::features2d::ORB>::create(
        500,
        1.2,
        8,
        31,
        0,
        2,
        opencv::features2d::ORB_ScoreType::HARRIS_SCORE,
        31,
        20,
    )?;

    let mut orb_keypoints = opencv::core::Vector::default();
    let mut orb_desc = opencv::core::Mat::default();
    let dst_image = opencv::core::Mat::default();
    let mask = opencv::core::Mat::default();

    orb.detect_and_compute(&image, &mask, &mut orb_keypoints, &mut orb_desc, false)?;

    // Use SIFT
    let mut sift = opencv::features2d::SIFT::create(0, 3, 0.04, 10., 1.6)?;
    let mut sift_keypoints = opencv::core::Vector::default();
    let mut sift_desc = opencv::core::Mat::default();

    sift.detect_and_compute(&image, &mask, &mut sift_keypoints, &mut sift_desc, false)?;

    // Write image using OpenCV
    opencv::imgcodecs::imwrite("./tmp.png", &dst_image, &opencv::core::Vector::default())?;

    // Convert :: cv::core::Mat -> ndarray::ArrayView3
    let a = dst_image.try_as_array()?;

    // Convert :: ndarray::ArrayView3 -> RgbImage
    // Note, this require copy as RgbImage will own the data
    let test_image = array_to_image(a);

    // Note, the colors will be swapped (BGR <-> RGB)
  	// Will need to swap the channels before
    // converting to RGBImage
    // But since this is only a demo that
    // it indeed works to convert cv::core::Mat -> ndarray::ArrayView3
    // I'll let it be
    test_image.save("out.png")?;

    Ok(())
}

trait AsArray {
    fn try_as_array(&self) -> Result<ArrayView3<u8>>;
}

impl AsArray for opencv::core::Mat {
    fn try_as_array(&self) -> Result<ArrayView3<u8>> {
        if !self.is_continuous() {
            return Err(anyhow!("Mat is not continuous"));
        }

        let bytes = self.data_bytes()?;
        let size = self.size()?;
        let a = ArrayView3::from_shape((size.height as usize, size.width as usize, 3), bytes)?;

        Ok(a)
    }
}

// From Stack Overflow: https://stackoverflow.com/questions/56762026/how-to-save-ndarray-in-rust-as-image
fn array_to_image(arr: ArrayView3<u8>) -> RgbImage {
    assert!(arr.is_standard_layout());

    let (height, width, _) = arr.dim();
    let raw = arr.to_slice().expect("Failed to extract slice from array");

    RgbImage::from_raw(width as u32, height as u32, raw.to_vec())
            .expect("container should have the right size for the image dimensions")
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
        let mut path = String::from("./");

        if args.len() >= 2 {
            path = args[1].clone();
        }

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
