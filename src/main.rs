#![feature(iter_intersperse)]
#![deny(clippy::pedantic)]
use image::{open, DynamicImage, GenericImage, GenericImageView, ImageFormat, ImageResult};

use std::env;

use crate::bit_iter::BitIter;

mod bit_iter;

/// An 48-bit (16 pixel) suffix for any message. This allows for detecting
/// where to stop reading.
/// In order to make this as unlikely to match with actual data as possible,
/// it uses high and low bits.
const LSB_MESSAGE_SUFFIX: &[u8; 6] = &[u8::MAX, u8::MAX, u8::MIN, u8::MIN, u8::MAX, u8::MIN];

/// Hide a message inside the least significant bit of each RGB-part
/// of a pixel. This means we can store 3 bits per pixel.
/// Bits are stored from high to low.
fn hide_lsb(image: &mut DynamicImage, message: &[u8]) {
    // Make sure we have enough space in the image to hide the message and the LSB suffix.
    debug_assert!(
        u8::BITS * (message.len() + LSB_MESSAGE_SUFFIX.len()) as u32
            <= image.width() * image.height() * 3
    );

    // Create an iterator over all bits of the message.
    let mut bits = message
        .iter()
        .chain(LSB_MESSAGE_SUFFIX)
        .map(|byte| (*byte).iter_bits())
        .flatten();
    let mut any_was_none = false;

    // Iterate the pixels of the image and merge the bits, if possible.
    for y in 0..image.height() {
        for x in 0..image.width() {
            let mut pixel = image.get_pixel(x, y);

            // Iterate the RGB channels and set the bits.
            for idx in 0..3 {
                if let Some(value) = bits.next() {
                    if value {
                        pixel.0[idx] |= 1;
                    } else {
                        pixel.0[idx] &= !1;
                    }
                } else {
                    any_was_none = true;
                    break;
                }
            }

            image.put_pixel(x, y, pixel);

            // Terminate early if there are no more bits to hide.
            if any_was_none {
                return;
            }
        }
    }
}

/// Reveal a message inside the least significant (8th) bit of each RGB-part
/// of a pixel.
/// Bits are stored from high to low.
fn reveal_lsb(image: &DynamicImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = 0;
    let mut bits_read = 0;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);

            for idx in 0..3 {
                byte |= (pixel.0[idx] & 1) << bits_read;
                bits_read += 1;

                if bits_read == 8 {
                    bytes.push(byte);
                    byte = 0;
                    bits_read = 0;

                    // Check if we got a magic suffix
                    if bytes.ends_with(LSB_MESSAGE_SUFFIX) {
                        bytes.truncate(bytes.len() - LSB_MESSAGE_SUFFIX.len());
                        return bytes;
                    }
                }
            }
        }
    }

    bytes
}

macro_rules! get_next_argument_or {
    ($params:expr, $to_print:expr) => {
        match $params.next() {
            Some(i) => i,
            None => {
                println!($to_print);
                return Ok(());
            }
        }
    };
}

macro_rules! might_fail {
    ($op:expr, $to_print:expr) => {
        match $op {
            Ok(i) => i,
            Err(_) => {
                println!($to_print);
                return Ok(());
            }
        }
    };
}

fn main() -> ImageResult<()> {
    // Parse command line arguments
    let mut params = env::args();

    let program_name = get_next_argument_or!(params, "Program name missing in invocation");
    let command = get_next_argument_or!(params, "No command provided, try `{program_name} help`");

    match command.as_str() {
        "help" => {
            println!(
                "\
{program_name} - Tiny tool for image steganography

Commands:
    `help` - shows this message
    `hide [in_file] [out_file] [message]` - hides a message in an image
    `reveal [in_file]` - reveals a message in an image"
            )
        }
        "hide" => {
            let in_file = get_next_argument_or!(
                params,
                "No input file provided. Usage: `{program_name} hide [in_file] [out_file]`"
            );
            let out_file = get_next_argument_or!(
                params,
                "No output file provided. Usage: `{program_name} hide [in_file] [out_file]`"
            );
            let message: String = params.intersperse(String::from(" ")).collect();

            let mut input = might_fail!(open(in_file), "Could not open input file");

            hide_lsb(&mut input, message.as_bytes());

            let format = might_fail!(
                ImageFormat::from_path(&out_file),
                "Image format for output file could not be determined"
            );
            might_fail!(
                input.save_with_format(out_file, format),
                "Could not save output file"
            );
        }
        "reveal" => {
            let in_file = get_next_argument_or!(
                params,
                "No input file provided. Usage: `{program_name} hide [in_file] [out_file]`"
            );

            let input = might_fail!(open(in_file), "Could not open input file");

            let value = reveal_lsb(&input);

            println!("{}", String::from_utf8_lossy(&value));
        }
        _ => println!("Unknown command, try `{program_name} help`"),
    }

    Ok(())
}
