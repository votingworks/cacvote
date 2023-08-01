use base64::{engine::general_purpose::STANDARD, Engine};
use color_eyre::eyre::eyre;
use rqrr::PreparedImage;

pub fn decode_page_from_image(image: image::GrayImage) -> color_eyre::Result<Vec<u8>> {
    let mut prepared_image = PreparedImage::prepare(image);

    prepared_image
        .detect_grids()
        .iter()
        .flat_map(|g| g.decode())
        .next()
        .map_or_else(
            || Err(eyre!("No QR code found")),
            |(_, content)| {
                STANDARD
                    .decode(content.as_str())
                    .map_err(|_| eyre!("Unable to decode QR code: {}", content))
            },
        )
}
