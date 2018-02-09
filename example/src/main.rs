#[macro_use] extern crate honggfuzz;

fn main() {
    // Here you can parse `std::env::args and 
    // setup / initialize your project

    // You have full control over the loop but
    // you're supposed to call `fuzz` ad vitam aeternam
    loop {
        // The fuzz macro gives an arbitrary object (see `arbitrary crate`)
        // to a closure-like block of code.
        // For performance, it is recommended that you use the native type
        // `&[u8]` when possible.
        // Here, this slice will contain a "random" quantity of "random" data.
        fuzz!(|data: &[u8]| {
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
    }
}
