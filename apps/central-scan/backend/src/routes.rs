use std::path::PathBuf;
use std::sync::mpsc::channel;

use rocket::response::stream::TextStream;

use central_scanner::scan;

use crate::cards::decode_page_from_image;

#[get("/")]
pub fn hello_world() -> &'static str {
    "Hello, world!"
}

#[get("/scan")]
pub fn do_scan() -> TextStream![String] {
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        let session = scan(PathBuf::from("/tmp")).unwrap();
        for (side_a_path, side_b_path) in session {
            tx.send((side_a_path, side_b_path)).expect("send() failed");
        }
    });

    TextStream! {
        for (side_a_path, side_b_path) in rx {
            yield format!("card: {:?}\n", rayon::join(
                move || {
                    let start = std::time::Instant::now();
                    let side_a_image = image::open(side_a_path).unwrap().to_luma8();
                    eprintln!("A opened in {:?}", start.elapsed());
                    let decoded = decode_page_from_image(side_a_image);
                    eprintln!("A decoded in {:?}", start.elapsed());
                    decoded
                },
                move || {
                    let start = std::time::Instant::now();
                    let side_b_image = image::open(side_b_path).unwrap().to_luma8();
                    eprintln!("side_b opened in {:?}", start.elapsed());
                    let decoded = decode_page_from_image(side_b_image);
                    eprintln!("side_b decoded in {:?}", start.elapsed());
                    decoded
                },
            ));
        }

        handle.join().unwrap();
    }
}
