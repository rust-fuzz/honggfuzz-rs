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
        // Here, this tuple will contain three "random" values of different type.
        fuzz!(|data: (bool, i32, f32)| {

            if data.0 == false {return}
            if data.1 < 0 {return}
            if data.2 > 1.0 {return}
            panic!("BOOM")

        });
    }
}
