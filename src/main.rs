use anyhow::{anyhow, Context};
use image::imageops::flip_vertical_in_place;
use image::{GenericImageView, Pixel};
use std::convert::{TryFrom, TryInto};

fn main() -> anyhow::Result<()> {
    if let Err(error) = real_main() {
        native_dialog::MessageDialog::new()
            .set_title("An error occurred")
            .set_text(error.to_string().as_ref())
            .show_alert()
            .context("oh now even the error handling dialog dies? a-maz-ing")?
    }
    Ok(())
}

fn real_main() -> anyhow::Result<()> {
    let path = native_dialog::FileDialog::new()
        .add_filter(
            "Image files",
            &[
                "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "avif", "pnm",
                "pam", "pbm", "pgm", "ppm", "dds", "tga", "exr", "farbfeld",
            ],
        )
        .show_open_single_file()
        .context("Failed to show file select dialog")?
        .ok_or_else(|| anyhow!("No file selected"))?;
    let dir = path
        .canonicalize()
        .context("Failed to canonicalize path; Did the folder disappear into thin air?")?
        .parent()
        .ok_or_else(|| anyhow!("Can't find the parent of a file???? Very weird"))?
        .join(
            path.file_stem()
                .ok_or_else(|| anyhow!("File name doesn't exist??? But we selected a file!"))?,
        );
    let dir = dir.as_path();
    let mut input = image::open(path).context("Failed to open image")?;
    flip_vertical_in_place(&mut input);
    let width = input.width();
    let height = input.height();
    let input = input.to_rgba8();
    let input: Vec<_> = input
        .enumerate_pixels()
        .map(|(_, _, pixel)| imagequant::RGBA::from(pixel.0))
        .collect();
    let mut attrs = imagequant::new();
    attrs
        .set_speed(1)
        .ok()
        .context("Failed to set speed option for image quantization")?;
    attrs
        .set_max_colors(11)
        .ok()
        .context("Failed to set max colors option for image quantization")?;
    let mut image = attrs
        .new_image(
            input.as_ref(),
            width.try_into().context("image too wide")?,
            height.try_into().context("image too tall")?,
            0.0,
        )
        .context("Failed to make quantization image object")?;
    let mut result = attrs.quantize(&image).context("Failed to quantize image")?;
    result
        .set_dithering_level(1.0)
        .ok()
        .context("Failed to set dithering option for image remapping")?;
    let (palette, arr) = result
        .remapped(&mut image)
        .context("Failed to remap image to palette")?;
    let mut colors = palette.iter();
    std::fs::create_dir(dir).context("Failed to create output directory")?;
    for name in &[
        "0.png",
        "BGDefault.png",
        "CCWspinner_alt.png",
        "CWspinner_alt.png",
        "enemy.png",
        "generator.png",
        "immobile.png",
        "mover.png",
        "push.png",
        "slide.png",
        "trash.png",
    ] {
        if let Some(color) = colors.next() {
            image::ImageBuffer::from_pixel(
                1,
                1,
                image::Rgba::from_channels(color.r, color.g, color.b, color.a),
            )
            .save(dir.join(name))
            .context(format!("Failed to save texture {}", name))?
        }
    }
    std::fs::write(dir.join("level.txt"), encode_v3_code(width, height, arr)?)
        .context("Failed to write level code")?;
    native_dialog::MessageDialog::new()
        .set_text(&*format!(
            "Your image was successfully converted!\
         The texture pack and the level code is available at: {}",
            dir.display()
        ))
        .show_alert()
        .context("Failed to create success dialog")?;
    Ok(())
}

fn encode_v3_code(mut width: u32, mut height: u32, data: Vec<u8>) -> anyhow::Result<String> {
    let mut result = "V3;".to_owned();
    let keys = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '!', '$', '%', '&', '+', '-', '.', '=', '?', '^',
        '{', '}',
    ];
    let cell_type_keys = ['}', '{', '4', '2', 'e', '0', 'c', '6', 'a', 'I', 'G'];
    let mut width_encoded = String::new();
    while width > 0 {
        width_encoded.push(
            *keys
                .get(usize::try_from(width % 74).context(
                    "Apparently usize can't hold a number smaller than 74? what the hell",
                )?)
                .ok_or_else(|| anyhow!("Who changed the array to be smaller"))?,
        );
        width /= 74;
    }
    result += &*width_encoded.chars().rev().collect::<String>();
    result += ";";
    let mut height_encoded = String::new();
    while height > 0 {
        height_encoded.push(
            *keys
                .get(usize::try_from(height % 74).context(
                    "Apparently usize can't hold a number smaller than 74? what the hell",
                )?)
                .ok_or_else(|| anyhow!("Who changed the array to be smaller"))?,
        );
        height /= 74;
    }
    result += &*height_encoded.chars().rev().collect::<String>();
    result += ";";
    for cell in data {
        result.push(*cell_type_keys.get(usize::from(cell)).ok_or_else(|| {
            anyhow!(
                "libimagequant didn't listen to me and quantized to >11 colors;\
                 Now I can't map the pixel to a cell"
            )
        })?)
    }
    Ok(result + ";")
}
