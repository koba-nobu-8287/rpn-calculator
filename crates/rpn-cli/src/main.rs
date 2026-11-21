use std::{env, io, process::ExitCode};
fn main() -> ExitCode {
    let args = env::args_os()
        .skip(1)
        .map(|v| v.into_string())
        .collect::<Result<Vec<_>, _>>();
    let mut stderr = io::stderr().lock();
    let result = match args {
        Ok(args) => rpn_cli::terminal::run(
            args,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
            &mut stderr,
        ),
        Err(_) => Err(rpn_cli::line::LineError::Syntax(
            rpn_cli::line::CliError::InvalidArgument("non-UTF-8 argument".into()),
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            use io::Write;
            if writeln!(stderr, "{e}").is_err() {
                return ExitCode::from(3);
            }
            ExitCode::from(e.exit_code())
        }
    }
}
