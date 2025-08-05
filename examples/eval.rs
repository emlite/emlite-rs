use emlite::{Console, eval};

fn main() {
    emlite::init();
    let con = Console::get();
    let ret = eval!(
        r#"
        let con = EMLITE_VALMAP.toValue({});
        con.log("Hello");
        6
    "#,
        con.as_handle()
    );
    con.log(&[ret]);
}
