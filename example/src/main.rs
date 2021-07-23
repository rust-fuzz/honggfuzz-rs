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
        // The fuzz macro gives a `&[u8]` by default, with an option to use the
        // `arbitrary crate` instead if you prefer.
        // For performance, it is recommended that you use the native type
        // `&[u8]` when possible.
        // Here, this slice will contain a "random" quantity of "random" data.
        #[cfg(not(feature = "arbitrary"))]
        fuzz!(|data: &[u8]| {
            // Try to access the global state across the unwind boundary
            some_global_state += 1;

            if data.len() != 3 {return}
            if data[0] != b'h' {return}
            if data[1] != b'e' {return}
            if data[2] != b'y' {return}
            panic!("BOOM")
        });
            
        // The fuzz macro gives an arbitrary object (see `arbitrary crate`)
        // to a closure-like block of code if you set the `arbitrary` feature.
        // Here, this tuple will contain two "random" values of different type.
        #[cfg(feature = "arbitrary")]
        fuzz!(|data: (i8, u16)| {
            if data.0 != b'h' as i8 {return}
            if data.1 & 0xff != b'e' as u16 {return}
            // If we check both bytes of `data.1` at the same time the compiler
            // optimizes it into a single check and the fuzzer has to run more
            // cycles in order to find the crash. By writing through a volatile
            // pointer in between the checks we ensure the compiler has to
            // check each byte separately, making it much easier for the fuzzer
            // to find the crash.
            unsafe { std::ptr::write_volatile(&mut some_global_state, 42); }
            if (data.1 >> 8) & 0xff != b'y' as u16 {return}
            panic!("BOOM")
        });
    }
}
