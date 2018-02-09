extern crate honggfuzz;
use honggfuzz::fuzz;

fn main() {
    // Here you can parse `std::env::args and 
    // setup / initialize your project

    // You have full control over the loop but
    // you're supposed to call `fuzz` ad vitam aeternam
    loop {
        // the fuzz function takes a closure which takes
        // a reference to a slice of u8.
        // This slice contains a "random" quantity of "random" data.
        fuzz(|data|{
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
