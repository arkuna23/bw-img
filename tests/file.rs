use std::{fs, io::Cursor};

use bw_img::{file::zip::{compress_imgs, decompress_imgs}, img::BWImageSize, BWImage, ImageData, NormalImage};

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
    let img = BWImage::parse_file(&mut buffer).unwrap().unwrap();
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
        NormalImage::new(&image::open("assets/ferries.png").unwrap())
            .parse_bw_image()
            .unwrap(),
        NormalImage::new(&image::open("assets/rust.png").unwrap())
            .parse_bw_image()
            .unwrap(),
    ];

    compress_imgs(&imgs, &mut fs::File::create("/tmp/test").unwrap()).unwrap();
    decompress_imgs(&mut fs::File::open("/tmp/test").unwrap()).unwrap();
}
