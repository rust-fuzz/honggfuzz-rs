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
            if data[0] != b'q' {return}
            if data[1] != b'w' {return}
            if data[2] != b'e' {return}
            if data[3] != b'r' {return}
            if data[4] != b't' {return}
            if data[5] != b'y' {return}
            if data[6] != b'u' {return}
            if data[7] != b'i' {return}
            if data[8] != b'o' {return}
            if data[9] != b'p' {return}
            panic!("BOOM")
        });
    }
}
