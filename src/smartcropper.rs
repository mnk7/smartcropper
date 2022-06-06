use image::{DynamicImage, GenericImage, GenericImageView, GrayImage};
use imageproc::rect::Rect;
use std::cmp::min;
use std::cmp::max;
use num_traits::cast::NumCast;

extern crate rustface;

use rustface::{Detector, FaceInfo, ImageData};
use crate::configuration::configuration;

pub fn run(configuration: configuration) -> Result<bool, String> {
    let image: DynamicImage = match image::open(configuration.path()) {
        Ok(image) => image,
        Err(error) => {
            let message = String::from("Failed to read image: ") + &String::from(error.to_string());
            return Err(message)
        }
    };

    // Decide weather to crop the height or width. We want to keep as much from the image as
    // possible, so only one has to be cut to match the aspect ratio.
    let mut crop_width = false;
    let mut crop_height = false;

    let mut new_width = image.width();
    let mut new_height = image.height();

    let image_original_aspect_ratio: f32 = image.width() as f32 / image.height() as f32;
    let image_new_aspect_ratio: f32 = configuration.width().clone() as f32
        / configuration.height().clone() as f32;

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
        };
        crop_width = true;
    } else {
        // the original height is higher than wanted
        new_height = match NumCast::from(image.width() as f32 / image_new_aspect_ratio) {
            Some(num_height) => num_height,
            None => {
                println!("could not compute new height");
                image.height()
            },
        };
        crop_height = true;
    }

    println!("crop image from size {}x{} to size {}x{}",
             image.width(), image.height(),
             &new_width, &new_height);

    let mut detector = match rustface::create_detector(configuration.model_path()) {
        Ok(detector) => detector,
        Err(error) => {
            let message = String::from("Failed to create detector: ") + &error.to_string();
            return Err(message)
        }
    };

    detector.set_min_face_size(max(image.width() / 100, 20));
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    let mut rgb = image.to_rgba8();
    let faces = detect_faces(&mut *detector, &image.to_luma8());

    // Get the rectangle that contains all detected faces in the image:
    struct Bounds {
        min_x: u32,
        max_x: u32,
        min_y: u32,
        max_y: u32,
    }
    
    let mut containing_bounds = Bounds {
        min_x: if crop_width { new_width.clone() / 2 } else { 0 },
        max_x: if crop_width { new_width.clone() / 2 } else { new_width.clone() },
        min_y: if crop_height { new_height.clone() / 2 } else { 0 },
        max_y: if crop_height { new_height.clone() / 2 } else { new_height.clone() }
    };

    for face in faces {
        let bbox = face.bbox();
        let rect = Rect::at(bbox.x(), bbox.y()).of_size(bbox.width(), bbox.height());
        
        if rect.left() < containing_bounds.min_x.try_into().unwrap() {
            containing_bounds.min_x = rect.left() as u32;
        }
        
        if rect.right() > containing_bounds.max_x.try_into().unwrap() {
            containing_bounds.max_x = rect.right() as u32;
        }

        if rect.top() < containing_bounds.min_y.try_into().unwrap() {
            containing_bounds.min_y = rect.top() as u32;
        }

        if rect.bottom() > containing_bounds.max_y.try_into().unwrap() {
            containing_bounds.max_y = rect.bottom() as u32;
        }
    }

    println!("detected faces inside rect from ({}, {}) to ({}, {})",
        &containing_bounds.min_x, &containing_bounds.min_y,
        &containing_bounds.max_x, &containing_bounds.max_y);

    // maximize the area of the cropped image around the containing bounds
    let mut x_of_crop_rectangle = (containing_bounds.min_x + containing_bounds.max_x) / 2;
    // move the frame inside the original image
    x_of_crop_rectangle = max(x_of_crop_rectangle, 0);
    x_of_crop_rectangle = min(x_of_crop_rectangle, image.width() - new_width.clone());

    let mut y_of_crop_rectangle = (containing_bounds.min_y + containing_bounds.max_y) / 2;
    // move the frame inside the original image
    y_of_crop_rectangle = max(y_of_crop_rectangle, 0);
    y_of_crop_rectangle = min(y_of_crop_rectangle, image.height() - new_height.clone());

    let crop_rectangle = Rect::at(x_of_crop_rectangle.try_into().unwrap(),
                                      y_of_crop_rectangle.try_into().unwrap()).of_size(
                                      new_width,
                                      new_height);

    rgb = rgb.sub_image(crop_rectangle.left() as u32,
                        crop_rectangle.top() as u32,
                        crop_rectangle.width(),
                        crop_rectangle.height())
        .to_image();

    match rgb.save(configuration.output_path()) {
        Ok(_) => println!("Saved result to {}", configuration.output_path()),
        Err(error) => {
            let message = String::from("Failed to save result to a file. Reason: ")
                + &String::from(error.to_string());
            return Err(message)
        }
    }

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
