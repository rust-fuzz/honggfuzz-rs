//! ## About Honggfuzz
//!
//! Honggfuzz is a security oriented fuzzer with powerful analysis options. Supports evolutionary, feedback-driven fuzzing based on code coverage (software- and hardware-based)
//!
//! * project homepage [honggfuzz.com](http://honggfuzz.com/)
//! * project repository [github.com/google/honggfuzz](https://github.com/google/honggfuzz)
//! * this upstream project is maintained by Google, but ...
//! * this is NOT an official Google product
//! 
//! ## How to use this crate
//!
//! Please see the [README.md](https://github.com/PaulGrandperrin/honggfuzz-rs/blob/master/README.md)
//! 
//! ## Relevant documentation about honggfuzz usage
//! * [USAGE](https://github.com/google/honggfuzz/blob/master/docs/USAGE.md)
//! * [FeedbackDrivenFuzzing](https://github.com/google/honggfuzz/blob/master/docs/FeedbackDrivenFuzzing.md)
//! * [PersistentFuzzing](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)
//! 

extern "C" {
    fn HF_ITER(buf_ptr: *mut *const u8, len_ptr: *mut usize );
}

pub fn fuzz<F>(closure: F) where F: Fn(&[u8]) {
    let buf;
    unsafe {
        let mut buf_ptr: *const u8 = std::mem::uninitialized();
        let mut len_ptr: usize = std::mem::uninitialized();
        HF_ITER(&mut buf_ptr, &mut len_ptr);
        buf = ::std::slice::from_raw_parts(buf_ptr, len_ptr);
    }
    closure(buf);
}

#[macro_export]
macro_rules! fuzz {
    (|$buf:ident| $body:block) => {
        honggfuzz::fuzz(|$buf| $body);
    };
    (|$buf:ident: &[u8]| $body:block) => {
        honggfuzz::fuzz(|$buf| $body);
    };
    (|$buf:ident: $dty: ty| $body:block) => {
        honggfuzz::fuzz(|$buf| {
            let $buf: $dty = {
                use arbitrary::{Arbitrary, RingBuffer};
                if let Ok(d) = RingBuffer::new($buf, $buf.len()).and_then(|mut b|{
                        Arbitrary::arbitrary(&mut b).map_err(|_| "")
                    }) {
                    d
                } else {
                    return
                }
            };

            $body
        });
    };
}