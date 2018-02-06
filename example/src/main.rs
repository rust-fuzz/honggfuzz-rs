#![no_main]
#[macro_use] extern crate honggfuzz;

fuzz_target!(|data: &[u8]| {
    if data.len() != 10 {return}
    if data[0] != 'q' as u8 {return}
    if data[1] != 'w' as u8 {return}
    if data[2] != 'e' as u8 {return}
    if data[3] != 'r' as u8 {return}
    if data[4] != 't' as u8 {return}
    if data[5] != 'y' as u8 {return}
    if data[6] != 'u' as u8 {return}
    if data[7] != 'i' as u8 {return}
    if data[8] != 'o' as u8 {return}
    if data[9] != 'p' as u8 {return}
    panic!("BOOM")
});

