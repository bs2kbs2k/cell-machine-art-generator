use dialog::DialogBox;
use image::imageops::flip_vertical_in_place;
use image::{GenericImageView, Pixel};
use std::convert::TryInto;

fn main() {
    let path = std::path::PathBuf::from(
        dialog::FileSelection::new("Select image")
            .mode(dialog::FileSelectionMode::Open)
            .show()
            .unwrap()
            .unwrap(),
    );
    let dir = path
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .join(path.file_stem().unwrap());
    let dir = dir.as_path();
    let mut input = image::open(path).unwrap();
    flip_vertical_in_place(&mut input);
    let width = input.width();
    let height = input.height();
    let input = input.to_rgba8();
    let input: Vec<_> = input
        .enumerate_pixels()
        .map(|(_, _, pixel)| imagequant::RGBA::from(pixel.0))
        .collect();
    let mut attrs = imagequant::new();
    attrs.set_speed(1).unwrap();
    attrs.set_max_colors(11).unwrap();
    let mut image = attrs
        .new_image(
            input.as_ref(),
            width.try_into().expect("image too wide"),
            height.try_into().expect("image too tall"),
            0.0,
        )
        .unwrap();
    let mut result = attrs.quantize(&image).unwrap();
    result.set_dithering_level(1.0);
    let (palette, arr) = result.remapped(&mut image).unwrap();
    let mut colors = palette.iter();
    std::fs::create_dir(dir).unwrap();
    for name in [
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
    ]
    .iter()
    {
        if let Some(color) = colors.next() {
            image::ImageBuffer::from_pixel(
                1,
                1,
                image::Rgba::from_channels(color.r, color.g, color.b, color.a),
            )
            .save(dir.join(name))
            .unwrap()
        }
    }
    std::fs::write(dir.join("level.txt"), encode_v3_code(width, height, arr)).unwrap();
    dialog::Message::new(format!(
        "Your image was successfully converted!\
         The texture pack and the level code is available at: {}",
        dir.display()
    ))
    .show()
    .unwrap();
}

fn encode_v3_code(mut width: u32, mut height: u32, data: Vec<u8>) -> String {
    let mut result = "V3;".to_string();
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
        width_encoded.push(keys[(width % 74) as usize]);
        width /= 74;
    }
    result += &*width_encoded.chars().rev().collect::<String>();
    result += ";";
    let mut height_encoded = String::new();
    while height > 0 {
        height_encoded.push(keys[(height % 74) as usize]);
        height /= 74;
    }
    result += &*height_encoded.chars().rev().collect::<String>();
    result += ";";
    for cell in data {
        result.push(cell_type_keys[cell as usize])
    }
    result + ";"
}
