use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
};

use image::{
    codecs::ico::{IcoEncoder, IcoFrame},
    ExtendedColorType, ImageReader,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input_dir = PathBuf::from(args.next().ok_or("missing input directory")?);
    let output_path = PathBuf::from(args.next().ok_or("missing output path")?);

    let icon_files = [
        "icon_16.png",
        "icon_32.png",
        "icon_64.png",
        "icon_128.png",
        "icon_256.png",
    ];

    let mut frames = Vec::with_capacity(icon_files.len());

    for icon_name in icon_files {
        let path = input_dir.join(icon_name);
        frames.push(load_frame(&path)?);
    }

    let file = File::create(output_path)?;
    IcoEncoder::new(file).encode_images(&frames)?;

    Ok(())
}

fn load_frame(path: &Path) -> Result<IcoFrame<'static>, Box<dyn std::error::Error>> {
    let image = ImageReader::open(path)?.decode()?.into_rgba8();
    let (width, height) = image.dimensions();
    let frame = IcoFrame::as_png(image.as_raw(), width, height, ExtendedColorType::Rgba8)?;
    Ok(frame)
}
