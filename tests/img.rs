use bw_img::{BWByteData, BWImage, IterDirection, IterOutput, NormalImage};

static RUST_BW: &[u8] = include_bytes!("../assets/rust.txt");
static RUST: &[u8] = include_bytes!("../assets/rust.png");

#[test]
fn img_rs_01() {
    let img = BWImage::parse(&NormalImage::new(&image::load_from_memory(RUST).unwrap())).unwrap();
    let mut out = String::new();
    for ele in img.iterator(IterDirection::Horizontal) {
        match ele {
            IterOutput::Byte(b) => {
                for ele in b.bw_byte_iter(8) {
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
