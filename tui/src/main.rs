use sdam::Sdam;

fn main() {
    let mut sdam=Sdam::new();

    let stdin=std::io::stdin();

    loop {
        let mut line=String::new();
        stdin.read_line(&mut line).unwrap();
        let line=line.trim().to_string();

        if line.contains("=") {
            let parts: Vec<&str>=line.split("=").collect();

            if parts.len()!=2 {
                continue;
                }

            let property: &str=&parts[0];
            let value: &str=&parts[1];

            match property {
                "rate" | "r" => {
                    let rate: f64=value.parse().unwrap();

                    sdam.set_rate(rate);
                    println!("Rate set");
                    },
                _ => {},
                }
            }
        else {
            match &line[..] {
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
    }
