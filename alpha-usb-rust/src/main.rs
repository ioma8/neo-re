use std::process::ExitCode;

fn main() -> ExitCode {
    match alpha_usb_rust::cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
