use honggfuzz::fuzz;

fn main() {
    // Here you can parse `std::env::args and
    // setup / initialize your project

    // You should avoid as much as possible global states.
    // This example is just there to test that this works nevertheless.
    // For more information, check out the `std::panic::UnwindSafe` trait.
    let mut some_global_state = 0u64;

    // You have full control over the loop but
    // you're supposed to call `fuzz` ad vitam aeternam
    loop {
        // The fuzz macro gives an arbitrary object (see `arbitrary crate`)
        // to a closure-like block of code.
        // For performance, it is recommended that you use the native type
        // `&[u8]` when possible.
        // Here, this slice will contain a "random" quantity of "random" data.
        fuzz!(|data: &[u8]| {
            // Try to access the global state across the unwind boundary
            some_global_state += 1;

            if data.len() != 3 {return}
            if data[0] != b'h' {return}
            if data[1] != b'e' {return}
            if data[2] != b'y' {return}
            panic!("BOOM")

        });
    }
}
