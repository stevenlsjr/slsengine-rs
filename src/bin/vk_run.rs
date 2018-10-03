extern crate slsengine;

use slsengine::sdl_platform::*;

fn main() {
    //    let plt = platform().with_window_size(640, 480)
//        .with_window_title("Hello rust!")
//        .with_vulkan()
//        .build().unwrap();
    use std::sync::mpsc;
    use std::thread;

    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let val = "hi".to_owned();
        sender.send(val).unwrap();
    });


    let received = receiver.recv().unwrap();
    println!("got {:?}", received);

    use std::io::{self, Read};
    if cfg!(windows) {
        let mut buffer = [0; 1];
        io::stdin().read(&mut buffer);
    }
}