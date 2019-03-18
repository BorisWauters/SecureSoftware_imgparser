extern crate sdl2;
#[macro_use] extern crate simple_error;

use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::prelude::*;
use std::io::{Seek, SeekFrom};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use shuteye::sleep;
use std::time::Duration;

#[derive(Clone)]
struct Pixel
{
    R: u32,
    G: u32,
    B: u32
}

struct Image
{
    width: u32,
    height: u32,
    pixels: Vec<Vec<Pixel>>
}

fn show_image(image: &Image)
{
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let display_mode = video_subsystem.current_display_mode(0).unwrap();

    let w = match display_mode.w as u32 > image.width {
        true => image.width,
        false => display_mode.w as u32
    };
    let h = match display_mode.h as u32 > image.height {
        true => image.height,
        false => display_mode.h as u32
    };
    
    let window = video_subsystem
        .window("Image", w, h)
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .unwrap();
    let black = sdl2::pixels::Color::RGB(0, 0, 0);

    let mut event_pump = sdl.event_pump().unwrap();
    // render image
        canvas.set_draw_color(black);
        canvas.clear();

        for r in 0..image.height {
            for c in 0..image.width {
                let pixel = &image.pixels[image.height as usize - r as usize - 1][c as usize];
                canvas.set_draw_color(Color::RGB(pixel.R as u8, pixel.G as u8, pixel.B as u8));
                canvas.fill_rect(Rect::new(c as i32, r as i32, 1, 1)).unwrap();
            }
        }
        
        canvas.present();

    'main: loop 
    {        
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        sleep(Duration::new(0, 250000000));
    }
    
}

fn read_num(cursor: &mut Cursor<Vec<u8>>) -> Result<u32, Box<std::error::Error>> {
    let mut v: Vec<u8> = vec![];
    let mut c: [u8; 1] = [0];

    // consume whitespace
    loop {
        cursor.read(&mut c)?;
        match &c {
            b" " | b"\t" | b"\n" => { },
            _ => { cursor.seek(std::io::SeekFrom::Current(-1)); break; }
        }
    }

    // read number
    loop {
        cursor.read(&mut c)?;
        match c[0] {
            b'0' ... b'9' => { v.push(c[0]); },
            b' ' | b'\t' | b'\n' => { cursor.seek(std::io::SeekFrom::Current(-1)); break; },
            _ => { bail!("Parse error") }
        }
    }

    let num_str = std::str::from_utf8(&v)?;
    let num = num_str.parse::<u32>()?;
    Ok(num)
}

fn decode_ppm_image(cursor: &mut Cursor<Vec<u8>>) -> Result<Image, Box<std::error::Error>> {
    let mut image = Image { 
        width: 0,
        height: 0,
        pixels: vec![]
    };

    // read header
    let mut c: [u8; 2] = [0; 2];
    cursor.read(&mut c)?;
    match &c {
        b"P6" => { },
        _ => { bail!("error") }
    }
    
    let w = read_num(cursor)?;
    let h = read_num(cursor)?;
    let cr = read_num(cursor)?;

    print!("width: {}, height: {}, color range: {}\n", w, h, cr);

	// TODO: Parse the image here

    let mut pxls:Vec<Vec<Pixel>> = vec![vec![]; h as usize];

    let mut buff: [u8; 3] = [0; 3];
    cursor.seek(std::io::SeekFrom::Current(1));
    for x in 0..w {
        let mut row: Vec<Pixel> = vec!();
        for y in 0..h {
            cursor.read(&mut buff)?;
            
            let px = Pixel {
                R: buff[0] as u32,
                G: buff[1] as u32,
                B: buff[2] as u32
            };
            pxls[y as usize].push(px);
        }
    }

    let mut pxl = Pixel {
        R: pxls[100][500].R as u32,
        G: pxls[100][500].G as u32,
        B: pxls[100][500].B as u32
    };

    image = Image {
        width: w,
        height: h,
        pixels: pxls
        //pixels: vec![vec![pxl; w as usize]; h as usize]
    };

    //print!("{},{}__", image.pixels.len(), image.pixels[0].len());
    //print!("{},{}__", pxls.len(), pxls[0].len());

    Ok(image)
}

fn main() 
{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Syntax: {} <filename>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let display = path.display();

    let mut file = match File::open(&path)    {
        Err(why) => panic!("Could not open file: {} (Reason: {})", 
            display, why.description()),
        Ok(file) => file
    };

    // read the full file into memory. panic on failure
    let mut raw_file = Vec::new();
    file.read_to_end(&mut raw_file).unwrap();

    // construct a cursor so we can seek in the raw buffer
    let mut cursor = Cursor::new(raw_file);
    let mut image = match decode_ppm_image(&mut cursor) {
        Ok(img) => img,
        Err(why) => panic!("Could not parse PPM file - Desc: {}", why.description()),
    };

    show_image(&image);
}