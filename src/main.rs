use std::io;

fn main() {
    for _i in 0..40 {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_n_bytes) => {
                match buffer.as_str().trim() {
                    // match &buffer as &str {
                    "uci" => {
                        println!("id name MyChessQL");
                        println!("id author Johnny La Rue");
                        println!("option name Hash type spin default 1 min 1 max 128");
                        println!("uciok");
                    },

                    "isready" => println!("readyok"),

                    "stop" => (),

                    "ucinewgame" => (),

                    "position fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1" => (),

                    "go movetime 3000" => {
                        println!("info currmove e7e5 currmovenumber 1");
                        println!("bestmove e7e5");
                    },

                    _ => println!("{} unknown", buffer),

                }

                // println!("{} bytes read", n);
                // println!("{}", buffer);
            }
            Err(e) => println!("Error {}", e),
        }
    }

}
