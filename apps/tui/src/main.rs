use crate::tui::main_loop;

mod tui;


fn main() -> std::io::Result<()> {
    main_loop()
        .map_err(|err| {
            match crossterm::terminal::disable_raw_mode() {
                Ok(_) => err,
                Err(e) => e,
            }
        })
}




