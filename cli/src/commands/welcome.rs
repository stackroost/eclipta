
use figlet_rs::FIGfont;

pub fn run_welcome() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("ECLIPTA").unwrap();
    println!("\x1b[35m{}\x1b[1m", figure);
    println!("\x1b[1;36mself-hosted observability platform\x1b[0m\n");
}
