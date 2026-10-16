//! The `pa3ds` executable: everything it does is `pa3ds::__main__`, where the tests reach it.

fn main() -> std::process::ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = pa3ds::update::offer(&argv) {
        return std::process::ExitCode::from(code as u8);
    }
    std::process::ExitCode::from(pa3ds::__main__::main(&argv) as u8)
}
