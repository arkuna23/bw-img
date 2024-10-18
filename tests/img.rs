use std::fs;

use bw_img::{iter_direction, BWByteData, BWImage, IterOutput, NormalImage};

static RUST_BW: &[u8] = include_bytes!("../assets/rust.txt");
static RUST: &[u8] = include_bytes!("../assets/rust.png");

#[test]
fn img_rs_horizontal() {
    let img = BWImage::parse(&NormalImage::new(&image::load_from_memory(RUST).unwrap())).unwrap();
    let mut out = String::new();
    for ele in img.iterator(iter_direction::Horizontal) {
        match ele {
            IterOutput::Byte { byte, len } => {
                for ele in byte.bw_byte_iter(len) {
                    if ele {
                        out.push_str("██");
                    } else {
                        out.push_str("  ");
                    }
                }
            }
            IterOutput::NewLine => out.push('\n'),
        }
    }
    assert_eq!(RUST_BW, out.as_bytes());
}

#[test]
fn img_rs_vertical() {
    let img = BWImage::parse(&NormalImage::new(&image::load_from_memory(RUST).unwrap())).unwrap();
    let mut out: Vec<Vec<bool>> = vec![];
    let mut current = vec![];
    for ele in img.iterator(iter_direction::Vertical) {
        match ele {
            IterOutput::Byte { byte, len } => {
                for ele in byte.bw_byte_iter(len) {
                    current.push(ele);
                }
            }
            IterOutput::NewLine => {
                out.push(current);
                current = vec![];
            }
        }
    }

    let mut rotated = String::new();
    let rows = out[0].len();
    println!();
    for col in 0..img.size.width as usize {
        (0..rows).for_each(|row| {
            rotated.push_str(if out[row][col] { "██" } else { "  " });
        });
        rotated.push('\n');
    }

    fs::write("/tmp/test", rotated).unwrap();
}
