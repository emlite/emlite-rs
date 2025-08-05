use emlite::{Console, argv};

fn main() {
    emlite::init();
    let con = Console::get();
    con.log(&argv!["Hello from Emlite!"]);
}
