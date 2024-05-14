use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Cli {
    pub directory: PathBuf,
}

impl Cli {
    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1);

        let mut out = Cli::default();

        while let Some(arg) = args.next() {
            if arg == "--directory" {
                out.directory = args.next().unwrap().parse().unwrap();
            }
        }

        out
    }
}
