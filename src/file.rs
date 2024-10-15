use crate::{img::BWImageConfig, BWImage};

const MAGIC_NUMBER: &[u8; 4] = b"BWIM";

/// Parse the header of bw img file
pub fn parse_header<R: std::io::Read>(read: &mut R) -> super::Result<BWImageConfig> {
    let mut header = [0u8; 16];
    read.read_exact(&mut header)
        .map_err(|e| super::BWError::FileHeader(e.to_string()))?;
    if &header[0..4] != MAGIC_NUMBER {
        return Err(super::BWError::FileHeader(
            "invalid magic number".to_string(),
        ));
    }
    if u32::from_le_bytes([header[4], header[5], header[6], header[7]]) != 1 {
        return Err(super::BWError::FileHeader(
            "invalid version number".to_string(),
        ));
    }
    Ok(BWImageConfig {
        width: u32::from_le_bytes([header[8], header[9], header[10], header[11]]),
        height: u32::from_le_bytes([header[12], header[13], header[14], header[15]]),
    })
}

/// write the header of bw img file
/// bw img file header format:
/// 0-3: magic number, "BWIM"
/// 4-7: version number, 1
/// 8-11: width, u32
/// 12-15: height, u32
pub fn write_header<W: std::io::Write>(
    write: &mut W,
    config: &BWImageConfig,
) -> std::io::Result<()> {
    write.write_all(MAGIC_NUMBER)?;
    write.write_all(&1u32.to_le_bytes())?;
    write.write_all(&config.width.to_le_bytes())?;
    write.write_all(&config.height.to_le_bytes())?;
    Ok(())
}

/// Parse the bw image from file
pub fn parse_file<R: std::io::Read>(input: &mut R) -> super::Result<BWImage> {
    let config = parse_header(input)?;
    let size = ((config.width * config.height) as f64 / 8f64).ceil() as usize;
    let mut data = vec![0u8; size];
    input.read_exact(&mut data)?;
    Ok(BWImage { config, data })
}

/// Encode the bw image to file
pub fn encode_file<W: std::io::Write>(output: &mut W, img: &BWImage) -> super::Result<()> {
    write_header(output, &img.config)?;
    output.write_all(&img.data)?;
    output.flush()?;
    Ok(())
}