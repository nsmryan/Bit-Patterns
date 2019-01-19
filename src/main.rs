extern crate gif;


use std::env;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::borrow::Cow;
use std::iter::Iterator;
use std::collections::HashMap;

use gif::{Frame, Encoder, Repeat, SetParameter};



#[derive(Clone)]
struct Bits<'a> {
    bytes: &'a Vec<u8>,
    bit_offset: usize,
}

impl<'a> Iterator for Bits<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let bit: bool;

        if self.bit_offset < self.bytes.len() * 8 {
            let byte = self.bytes[self.bit_offset / 8];
            bit = (byte & (1 << (self.bit_offset % 8))) != 0;

            self.bit_offset += 1;

            Some(bit)
        } else {
            None
        }
    }
}

impl<'a> Bits<'a> {
    fn new(bytes: &'a Vec<u8>) -> Self {
        Bits { bytes: &bytes, bit_offset: 0 }
    }
}


fn main() {
    if env::args().len() != 2 {
        println!("bit_patterns FILENAME]");
        return;
    }
    let file_name = &env::args().collect::<Vec<String>>()[1];

    let max_bit_width: u16 = 16;
    let size: u16 = 2_u16.pow(max_bit_width as u32 / 2);
    let width:  u16 = size as u16;
    let height: u16 = size as u16;

    let color_map = &[0xFF, 0xFF, 0xFF, 0, 0, 0];
    let mut byte_vec = Vec::new();

    let mut file = File::open(file_name).unwrap();

    file.read_to_end(&mut byte_vec);

    let bits = Bits::new(&byte_vec);

    let num_bits = bits.clone().count();

    let percent_ones: f64 = bits.clone().map(|bit| if bit { 1.0 } else { 0.0 } ).sum::<f64>() / num_bits as f64;

    let mut out_file_name = file_name.clone();
    out_file_name.push_str(".gif");
    let mut image = File::create(out_file_name).unwrap();
    let mut encoder = Encoder::new(&mut image, width, height, color_map).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    for bit_width in (2..max_bit_width+1).step_by(2) {
        let mut pixels = Vec::new();


        let mut bit_pattern_map: HashMap<u16, u32> = HashMap::new();
        for bit_pattern in 0..2_u32.pow(bit_width as u32) {
            bit_pattern_map.insert(bit_pattern as u16, 0);
        }

        println!("bit width = {}", bit_width);
        bits.clone().fold((0, 0), |(bit_pattern, index), b| {
            let mut bit_pattern = bit_pattern;
            let mut index = index;

            let bit_val = if b { 1 } else { 0 };
            bit_pattern |= bit_val << index;

            index += 1;
            if index == bit_width {
                let mut count = bit_pattern_map.get_mut(&bit_pattern).unwrap();
                *count = *count + 1;

                index = 0;
                bit_pattern = 0;
            }

            (bit_pattern, index)
        });

        let max_count = *bit_pattern_map.values().max().unwrap();

        let num_bit_patterns = 2_u32.pow(bit_width as u32);
        let divisor = ((width as usize * height as usize) / num_bit_patterns as usize) as u16;

        let size_used: u16 = 2_u16.pow(bit_width as u32 / 2);
        let col_divisor = width  / size_used;
        let row_divisor = height / size_used;

        for row in 0..height as usize {
            for col in 0..width as usize {
                let index = col + row * width as usize;
                let bit_pattern = (index / divisor as usize) as u16;
                let bit_pattern = ((col / col_divisor as usize) | ((row / row_divisor as usize) << (bit_width / 2))) as u16;

                //let mut bit_pattern: u16;
                //bit_pattern  = col as u16 / col_divisor;
                //bit_pattern |= (row as u16 / row_divisor) << (bit_width / 2);

                let mut count = *bit_pattern_map.get(&bit_pattern).unwrap() as f64;
                count = count.log(2.0);

                let color = (0xFF as f64 * count / (max_count as f64).log(2.0)) as u8;
                //if color != 0 {
                //    println!("({}, {}) = {:b} as {}", row, col, bit_pattern, color);
                //}


                pixels.push(0x00);
                pixels.push(color);
                pixels.push(0xFF);
            }
        }

        let mut frame = Frame::from_rgb(width, height, &pixels[..]);
        frame.delay = 200;
        encoder.write_frame(&frame).unwrap();
    }
}

