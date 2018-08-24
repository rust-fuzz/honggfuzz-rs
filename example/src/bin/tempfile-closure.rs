extern crate honggfuzz;

extern crate libc;
extern crate positioned_io;
extern crate tempfile;

use positioned_io::WriteAt;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::os::unix::io::AsRawFd;


fn main() {
    let mut df: File = tempfile::tempfile().unwrap();
    loop {
        honggfuzz::fuzz(|data: &[u8]| {
            let err = unsafe {
                libc::ftruncate(df.as_raw_fd(), 0)
            };
            assert!(err == 0, "Failed to truncate");
            df.write_all_at(0, data).expect("Failed to write");
            df.seek(SeekFrom::Start(0)).expect("Failed to seek");
            let mut data = Vec::new();
            df.read_to_end(&mut data)
                .expect("Failed to read");
            if data.len() != 6 {return}
            if data[0] != b'q' {return}
            if data[1] != b'w' {return}
            if data[2] != b'e' {return}
            if data[3] != b'r' {return}
            if data[4] != b't' {return}
            if data[5] != b'y' {return}
            panic!("BOOM")
        });
    }
}

