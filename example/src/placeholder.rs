use rand::Rng;
use std::fs::File;
use std::io::Read;
use texture_synthesis as ts;
use texture_synthesis::image;
use texture_synthesis::image::{imageops::FilterType, png::PNGEncoder, ColorType, DynamicImage};

pub(crate) fn mix_image() -> String {
    let placeholders = &[
        &"static/placeholder-1.png",
        &"static/placeholder-2.png",
        &"static/placeholder-3.png",
        &"static/placeholder-4.png",
        &"static/placeholder-5.png",
    ];

    const IMG_X: usize = 256;
    const IMG_Y: usize = 160;
    let mut rng = rand::thread_rng();
    let slices: usize = rng.gen_range(1, 5);
    const IMG_LEN: usize = IMG_X * IMG_Y * 3;
    let slice_len = IMG_LEN / slices;
    let mut img_buff = Vec::new();
    for _ in 0..slices {
        let idx: usize = rng.gen_range(0, 4);
        let mut f = File::open(placeholders[idx]).unwrap();
        let mut buffer = vec![0_u8; slice_len];
        let n = f.read(buffer.as_mut_slice()).unwrap();
        buffer.truncate(n as usize);
        // log::debug!("trunc buff len {}", buffer.len());
        img_buff.extend(buffer);
    }
    let pixels = img_buff.len() / 3;
    let sl_y = (10_f64 * pixels as f64).sqrt() / 4_f64;
    let sl_x = pixels as f64 / sl_y;
    let sl_buf = image::ImageBuffer::from_raw(sl_x as u32, sl_y as u32, img_buff);
    let sl_img = DynamicImage::ImageRgb8(sl_buf.unwrap());
    let sized_sl = sl_img.resize(IMG_X as u32, IMG_Y as u32, FilterType::Nearest);

    let mut img_buff = sized_sl.to_bytes();
    for _ in img_buff.len()..IMG_LEN {
        // img_buff.push(rand::random::<u8>())
        img_buff.push(0_u8)
    }

    let mut img_bytes = Vec::new();
    PNGEncoder::new(&mut img_bytes)
        .encode(&img_buff, IMG_X as u32, IMG_Y as u32, ColorType::Rgb8)
        .unwrap();
    return base64::encode(img_bytes);
}

pub(crate) fn generate_random_upscale() -> String {
    // 256 width 160 height and 3 color channels RGB each color is u8
    const IMG_X: usize = 256;
    const IMG_Y: usize = 160;
    const IMG_LEN: usize = IMG_X * IMG_Y * 3;
    let mut img_buff = Vec::new();
    let m_y = 5;
    let m_x = 8;
    for _ in 0..m_x {
        let mut r = rand::random::<u8>();
        let mut g = rand::random::<u8>();
        let mut b = rand::random::<u8>();
        if rand::random::<bool>() {
            r = 255;
            g = 255;
            b = 255;
        }
        for _ in 0..m_y {
            img_buff.push(r);
            img_buff.push(g);
            img_buff.push(b);
        }
    }
    let m_buf = image::ImageBuffer::from_raw(m_x as u32, m_y as u32, img_buff);
    let mini_img = DynamicImage::ImageRgb8(m_buf.unwrap());
    let sized_mini = mini_img.resize(IMG_X as u32, IMG_Y as u32, FilterType::Nearest);

    let mut img_buff = sized_mini.to_bytes();
    for _ in img_buff.len()..IMG_LEN {
        img_buff.push(255_u8)
    }

    let mut img_bytes = Vec::new();
    PNGEncoder::new(&mut img_bytes)
        .encode(&img_buff, IMG_X as u32, IMG_Y as u32, ColorType::Rgb8)
        .unwrap();
    return base64::encode(img_bytes);
}

pub(crate) fn generate_random_image() -> String {
    // 256 width 160 height and 3 color channels RGB each color is u8
    const IMG_LEN: usize = 256 * 160 * 3;
    let mut img_buff = vec![];
    for _ in img_buff.len()..IMG_LEN {
        img_buff.push(rand::random::<u8>())
    }
    let mut img_bytes = Vec::new();
    PNGEncoder::new(&mut img_bytes)
        .encode(&img_buff, 256, 160, ColorType::Rgb8)
        .unwrap();
    return base64::encode(img_bytes);
}

pub(crate) fn generate_fractal_image() -> String {
    let imgx = 256;
    let imgy = 160;

    let scalex = 3.0 as f32 / imgx as f32;
    let scaley = 3.0 as f32 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);
    }

    // A redundant loop to demonstrate reading image data
    for x in 0..imgx {
        for y in 0..imgy {
            let cx = y as f32 * scalex - 1.5;
            let cy = x as f32 * scaley - 1.5;

            let c = num_complex::Complex::new(-0.4, 0.6);
            let mut z = num_complex::Complex::new(cx, cy);

            let mut i = 0;
            while i < 255 && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }

            let pixel = imgbuf.get_pixel_mut(x, y);
            let data = (*pixel as image::Rgb<u8>).0;
            *pixel = image::Rgb([data[0], i as u8, data[2]]);
        }
    }

    let mut img_bytes = Vec::new();
    PNGEncoder::new(&mut img_bytes)
        .encode(&imgbuf, 256, 160, ColorType::Rgb8)
        .unwrap();
    return base64::encode(img_bytes);
}

pub(crate) fn generate_image() -> String {
    //create a new session
    if let Ok(texsynth) = ts::Session::builder()
        //load a single example image
        .add_examples(&[
            &"static/placeholder-1.png",
            &"static/placeholder-2.png",
            &"static/placeholder-3.png",
            &"static/placeholder-4.png",
            &"static/placeholder-5.png",
        ])
        // .resize_input(ts::Dims {
        //     width: 400,
        //     height: 250,
        // })
        .output_size(ts::Dims {
            width: 256,
            height: 160,
        })
        .seed(rand::random())
        .build()
    {
        //generate an image
        let generated = texsynth.run(None);
        let img = generated.into_image();
        let mut img_bytes = Vec::new();
        PNGEncoder::new(&mut img_bytes)
            .encode(&img.to_bytes(), 256, 160, ColorType::Rgba8)
            .unwrap();
        return base64::encode(img_bytes);
    }
    "".to_owned()
}

#[cfg(test)]
mod tests {
    use super::generate_fractal_image;

    #[test]
    fn test_gen_img() {
        let b64_img = generate_fractal_image();
        let html_file = format!(
            "<!DOCTYPE html>
        <html>
        <head>
        <title>Title of the document</title>
        </head>
        <body>
        <div>
            <p>From wikipedia</p>
            <img src=\"data:image/png;base64,{}\" alt=\"Generated image\" />
        </div>
        </body>
        </html>",
            b64_img
        );
        std::fs::write("test.html", html_file).unwrap();
    }
}
