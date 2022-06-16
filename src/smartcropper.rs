use image::{DynamicImage, GenericImage, GenericImageView, GrayImage};
use imageproc::rect::Rect;
use std::cmp::min;
use std::cmp::max;
use std::result::Result;
use std::path::Path;
use std::ffi::OsStr;
use num_traits::cast::NumCast;

extern crate rustface;

use rustface::{Detector, FaceInfo, ImageData};
use crate::Configuration;

// Get the rectangle that contains all detected faces in the image:
struct Bounds {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

pub fn run(configuration: Configuration) -> Result<bool, String> {
    let output_path = match configuration.output_path {
        Some(output_path) => output_path,
        None => "./".to_string(),
    };

    let rustface_model = match configuration.rustface_model {
        Some(model) => model,
        None => {
            return Err("no model initialized.".to_string())
        }
    };

    let mut i = 0;

    for path in configuration.images_in_directory {
        let image: DynamicImage = match image::open(path.clone()) {
            Ok(image) => image,
            Err(error) => {
                let message = String::from("Failed to read image: ") + &String::from(error.to_string());
                continue
            }
        };

        let file_path = Path::new(&path);
        let fallback_name = format!("{}.png", &i);
        let file_name = file_path.file_name().unwrap_or(OsStr::new(&fallback_name))
            .to_str().unwrap_or(fallback_name.as_str());

        let temporary_save_path = Path::new(&output_path).join(&file_name);
        let save_path = match temporary_save_path.to_str() {
            Some(save_path) => save_path,
            None => fallback_name.as_str(),
        };

        // Decide weather to crop the height or width. We want to keep as much from the image as
        // possible, so only one has to be cut to match the aspect ratio.
        let mut crop_width = false;
        let mut crop_height = false;

        let mut new_width = image.width() as i32;
        let mut new_height = image.height() as i32;

        let image_original_aspect_ratio: f32 = image.width() as f32 / image.height() as f32;
        let image_new_aspect_ratio: f32 = configuration.width.clone() as f32
            / configuration.height.clone() as f32;

        println!("convert from aspect ratio {} to aspect ratio {}",
                 &image_original_aspect_ratio,
                 &image_new_aspect_ratio);

        if image_original_aspect_ratio > image_new_aspect_ratio {
            // the original width is wider than wanted
            new_width = match NumCast::from(image.height() as f32 * image_new_aspect_ratio) {
                Some(num_width) => num_width,
                None => {
                    println!("could not compute new height.");
                    image.width()
                },
            } as i32;
            crop_width = true;
        } else {
            // the original height is higher than wanted
            new_height = match NumCast::from(image.width() as f32 / image_new_aspect_ratio) {
                Some(num_height) => num_height,
                None => {
                    println!("could not compute new height");
                    image.height()
                },
            } as i32;
            crop_height = true;
        }

        println!("crop image from size {}x{} to size {}x{}",
                 image.width(), image.height(),
                 &new_width, &new_height);

        let mut detector = rustface::create_detector_with_model(rustface_model.clone());

        detector.set_min_face_size(max(image.width() / 100, 20));
        detector.set_score_thresh(2.0);
        detector.set_pyramid_scale_factor(0.8);
        detector.set_slide_window_step(4, 4);

        let mut rgb = image.to_rgba8();
        let faces = detect_faces(&mut *detector, &image.to_luma8());

        let mut containing_bounds = Bounds {
            min_x: if crop_width { image.width() as i32 } else { 0 },
            max_x: if crop_width { 0 } else { image.width() as i32 },
            min_y: if crop_height { image.height() as i32 } else { 0 },
            max_y: if crop_height { 0 } else { image.height() as i32 }
        };

        for face in faces {
            let bbox = face.bbox();
            let rect = Rect::at(bbox.x(), bbox.y()).of_size(bbox.width(), bbox.height());

            println!("detected face in rectangle of size ({}, {}) at: {}, {}",
                     &rect.width(), &rect.height(), &rect.left(), &rect.top());

            if rect.left() < containing_bounds.min_x {
                containing_bounds.min_x = rect.left().clone();
            }

            if rect.right() > containing_bounds.max_x {
                containing_bounds.max_x = rect.right().clone();
            }

            if rect.top() < containing_bounds.min_y {
                containing_bounds.min_y = rect.top().clone();
            }

            if rect.bottom() > containing_bounds.max_y {
                containing_bounds.max_y = rect.bottom().clone();
            }
        }

        println!("\tdetected faces inside rect from ({}, {}) to ({}, {})",
                 &containing_bounds.min_x, &containing_bounds.min_y,
                 &containing_bounds.max_x, &containing_bounds.max_y);

        // maximize the area of the cropped image around the feature-containing bounds
        let mut x_of_crop_rectangle: i32 = (containing_bounds.min_x + containing_bounds.max_x
            - new_width.clone()) / 2;
        // move the frame inside the original image
        x_of_crop_rectangle = max(x_of_crop_rectangle, 0);
        x_of_crop_rectangle = min(x_of_crop_rectangle, image.width() as i32 - new_width.clone());

        // assume that heads are mostly at the top of a body -> place them in the top of the picture
        let mut y_of_crop_rectangle: i32 = containing_bounds.min_y - (new_height.clone() / 5);
        // move the frame inside the original image
        y_of_crop_rectangle = max(y_of_crop_rectangle, 0);
        y_of_crop_rectangle = min(y_of_crop_rectangle, image.height() as i32 - new_height.clone());

        let crop_rectangle = Rect::at(x_of_crop_rectangle, y_of_crop_rectangle)
            .of_size(new_width as u32, new_height as u32);

        println!("cropping with top left corner ({}, {})", &x_of_crop_rectangle, &y_of_crop_rectangle);

        rgb = rgb.sub_image(crop_rectangle.left() as u32,
                            crop_rectangle.top() as u32,
                            crop_rectangle.width(),
                            crop_rectangle.height())
            .to_image();

        match rgb.save(&save_path) {
            Ok(_) => println!("Saved result to {}", &save_path),
            Err(error) => {
                let message = String::from("Failed to save result to a file. Reason: ")
                    + &String::from(error.to_string());
                continue
            }
        }

        i += 1;

        println!("finished cropping file number {}\n", &i);
    }

    println!("finished cropping\n");

    Ok(true)
}

fn detect_faces(detector: &mut dyn Detector, gray: &GrayImage) -> Vec<FaceInfo> {
    let (width, height) = gray.dimensions();
    let mut image = ImageData::new(gray, width, height);
    let faces = detector.detect(&mut image);
    println!(
        "Found {} faces",
        faces.len(),
    );

    faces
}
