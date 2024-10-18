use std::io::Cursor;

use bw_img::{
    file::compress::{compress_imgs, decompress_imgs},
    img::BWImageSize,
    BWImage, ImageData, NormalImage,
};

static RUST: &[u8] = include_bytes!("../assets/rust.png");
static FERRIES: &[u8] = include_bytes!("../assets/ferries.png");

#[test]
fn encode_and_parse() {
    let mut buffer = Cursor::new(Vec::new());
    BWImage {
        size: BWImageSize {
            width: 1,
            height: 1,
        },
        pixels: vec![0],
    }
    .encode_as_file(&mut buffer)
    .unwrap();

    buffer.set_position(0);
    let (img, _) = BWImage::parse_file(&mut buffer).unwrap().unwrap();
    assert_eq!(
        img.size,
        BWImageSize {
            width: 1,
            height: 1,
        }
    );
    assert_eq!(img.pixels, vec![0]);
}

#[test]
fn compress_and_decompress() {
    let imgs = [
        NormalImage::new(&image::load_from_memory(FERRIES).unwrap())
            .parse_bw_image()
            .unwrap(),
        NormalImage::new(&image::load_from_memory(RUST).unwrap())
            .parse_bw_image()
            .unwrap(),
    ];
    let mut buf = Vec::new();
    println!(
        "img height: {}, width {}",
        imgs[0].size.height, imgs[0].size.width
    );
    println!(
        "width after divided: {}",
        imgs[0].pixels.len() * 8 / imgs[0].size.height as usize
    );
    assert_eq!(
        imgs[0].size.get_padded_bytes_len(),
        imgs[0].pixels.len() as u64
    );
    assert_eq!(
        imgs[1].size.get_padded_bytes_len(),
        imgs[1].pixels.len() as u64
    );

    compress_imgs(&imgs, &mut buf).unwrap();
    let imgs = decompress_imgs(&mut Cursor::new(buf))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(imgs.len(), 2);
}
