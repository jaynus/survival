use amethyst::renderer::sprite::{SpriteList, SpritePosition};
use image::{self, GenericImageView};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn get_tile_size(filepart: &str) -> usize {
    // Last 2 digits of a filepart should be the size right?!
    let size_str = &filepart[filepart.len() - 2..filepart.len()];
    size_str.parse::<usize>().unwrap()
}

fn main() -> Result<(), std::io::Error> {
    // one possible implementation of walking a directory only visiting files

    return Ok(());

    let app_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = app_root.join("../resources/assets/spritesheets");

    for entry in fs::read_dir(path).unwrap() {
        match entry {
            Ok(file) => {
                let input_path = file.path();
                if input_path.is_file()
                    && match input_path.extension() {
                        Some(e) => e == "png",
                        None => false,
                    }
                {
                    println!("rerun-if-changed={}", file.path().to_string_lossy());

                    match image::open(input_path.as_path()) {
                        Ok(img) => {
                            // Extract the dimensions from the filename
                            let stride = get_tile_size(
                                input_path.as_path().file_stem().unwrap().to_str().unwrap(),
                            );

                            let mut sprites = Vec::new();

                            for x in (0..img.dimensions().0).step_by(stride) {
                                for y in (0..img.dimensions().1).step_by(stride) {
                                    sprites.push(SpritePosition {
                                        offsets: None,
                                        x,
                                        y,
                                        width: stride as u32,
                                        height: stride as u32,
                                        flip_horizontal: false,
                                        flip_vertical: false,
                                    })
                                }
                            }

                            let sheet = SpriteList {
                                texture_width: img.dimensions().0,
                                texture_height: img.dimensions().1,
                                sprites,
                            };

                            let s = ron::ser::to_string_pretty(
                                &sheet,
                                ron::ser::PrettyConfig::new()
                                    .with_depth_limit(10)
                                    .with_separate_tuple_members(false)
                                    .with_enumerate_arrays(false)
                                    .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME),
                            )
                            .expect("Serialization failed");

                            let output_path = input_path.as_path().with_extension("ron");
                            let mut file = std::fs::File::create(output_path.as_path()).unwrap();

                            file.write_all(s.as_bytes()).unwrap();
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}
