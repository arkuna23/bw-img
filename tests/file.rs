use std::io::Cursor;

use bw_img::{img::BWImageConfig, BWImage};

#[test]
fn encode_and_parse() {
    let mut buffer = Cursor::new(Vec::new());
    BWImage {
        config: BWImageConfig {
            width: 1,
            height: 1,
        },
        data: vec![0],
    }
    .encode_as_file(&mut buffer)
    .unwrap();

    buffer.set_position(0);
    let img = BWImage::parse_file(&mut buffer).unwrap();
    assert_eq!(
        img.config,
        BWImageConfig {
            width: 1,
            height: 1,
        }
    );
    assert_eq!(img.data, vec![0]);
}
