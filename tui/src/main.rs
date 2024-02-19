use sdam::Sdam;

fn main() {
    let mut sdam=Sdam::new();

    let stdin=std::io::stdin();

    loop {
        let mut line=String::new();
        stdin.read_line(&mut line).unwrap();

        match line.trim() {
            "record start" | "r" => sdam.start_recording(),
            "record stop" | "rs" => sdam.stop_recording(),
            "play" | "p" => sdam.play(),
            "toggle" | "t" => sdam.toggle_playback(),
            "forward" | "f" => sdam.forward(5),
            "backward" | "b" => sdam.backward(5),
            "quit" | "q" => break,
            _ => {},
            }
        }
    }
