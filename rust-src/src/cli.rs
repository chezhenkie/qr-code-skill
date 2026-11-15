//! Command line parsing, modelled on argparse: prefixes, clusters, exit codes.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE_TAIL: &str =
    "[-h] [-f FILE] [-o OUT] [--scale SCALE] [--border BORDER] [--html] [--title TITLE] [--term] [text]";
const WIDTH: usize = 80;

#[derive(Debug, PartialEq)]
pub struct Args {
    pub text: Option<String>,
    pub file: Option<String>,
    pub out: String,
    pub scale: u32,
    pub border: u32,
    pub html: bool,
    pub title: String,
    pub term: bool,
}

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Run(Args),
    Help,
    Version,
    Usage(String),
}

#[derive(Clone, Copy)]
enum Opt {
    Help,
    Version,
    File,
    Out,
    Scale,
    Border,
    Html,
    Title,
    Term,
}

impl Opt {
    fn display(self) -> &'static str {
        match self {
            Opt::Help => "-h/--help",
            Opt::Version => "--version",
            Opt::File => "-f/--file",
            Opt::Out => "-o/--out",
            Opt::Scale => "--scale",
            Opt::Border => "--border",
            Opt::Html => "--html",
            Opt::Title => "--title",
            Opt::Term => "--term",
        }
    }

    fn takes_value(self) -> bool {
        !matches!(
            self,
            Opt::Help | Opt::Version | Opt::Html | Opt::Term
        )
    }
}

/// Registration order decides how argparse lists ambiguous prefixes and wraps help.
const LONG_OPTIONS: [(&str, Opt); 9] = [
    ("--help", Opt::Help),
    ("--file", Opt::File),
    ("--out", Opt::Out),
    ("--scale", Opt::Scale),
    ("--border", Opt::Border),
    ("--html", Opt::Html),
    ("--title", Opt::Title),
    ("--term", Opt::Term),
    ("--version", Opt::Version),
];

struct State {
    text: Option<String>,
    file: Option<String>,
    out: String,
    scale: i64,
    border: i64,
    html: bool,
    title: String,
    term: bool,
    out_given: bool,
    scale_given: bool,
    title_given: bool,
    extras: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        State {
            text: None,
            file: None,
            out: "qr.png".to_string(),
            scale: 10,
            border: 4,
            html: false,
            title: "qr code".to_string(),
            term: false,
            out_given: false,
            scale_given: false,
            title_given: false,
            extras: Vec::new(),
        }
    }
}

impl State {
    fn positional(&mut self, token: &str) {
        if self.text.is_none() {
            self.text = Some(token.to_string());
        } else {
            self.extras.push(token.to_string());
        }
    }
}

enum Matched {
    Option(Opt, Option<String>),
    Unknown(String),
}

pub fn parse(argv: &[String]) -> Outcome {
    let mut state = State::default();
    let mut index = 0;
    let mut positional_only = false;
    while index < argv.len() {
        let token = &argv[index];
        index += 1;
        if positional_only {
            state.positional(token);
            continue;
        }
        if token == "--" {
            positional_only = true;
            continue;
        }
        if token == "-" || !token.starts_with('-') {
            state.positional(token);
            continue;
        }
        let matched = if token.starts_with("--") {
            parse_long(token, argv, &mut index)
        } else if is_negative_number(token) {
            state.positional(token);
            continue;
        } else {
            parse_short(token, argv, &mut index)
        };
        match matched {
            Ok(Matched::Option(opt, value)) => {
                if let Some(outcome) = apply(&mut state, opt, value) {
                    return outcome;
                }
            }
            Ok(Matched::Unknown(token)) => state.extras.push(token),
            Err(message) => return Outcome::Usage(message),
        }
    }
    finish(state)
}

fn parse_long(token: &str, argv: &[String], index: &mut usize) -> Result<Matched, String> {
    let (name, inline) = match token.find('=') {
        Some(pos) => (&token[..pos], Some(token[pos + 1..].to_string())),
        None => (token, None),
    };
    let opt = if let Some((_, opt)) = LONG_OPTIONS.iter().find(|(long, _)| *long == name) {
        *opt
    } else {
        let hits: Vec<&str> = LONG_OPTIONS
            .iter()
            .filter(|(long, _)| long.starts_with(name))
            .map(|(long, _)| *long)
            .collect();
        match hits.len() {
            0 => return Ok(Matched::Unknown(token.to_string())),
            1 => {
                match LONG_OPTIONS.iter().find(|(long, _)| *long == hits[0]) {
                    Some((_, opt)) => *opt,
                    None => return Ok(Matched::Unknown(token.to_string())),
                }
            }
            _ => {
                return Err(format!(
                    "ambiguous option: {} could match {}",
                    name,
                    hits.join(", ")
                ))
            }
        }
    };
    if !opt.takes_value() {
        if let Some(value) = inline {
            return Err(format!(
                "argument {}: ignored explicit argument '{}'",
                opt.display(),
                value
            ));
        }
        return Ok(Matched::Option(opt, None));
    }
    match inline {
        Some(value) => Ok(Matched::Option(opt, Some(value))),
        None => match next_value(argv, index) {
            Some(value) => Ok(Matched::Option(opt, Some(value))),
            None => Err(format!(
                "argument {}: expected one argument",
                opt.display()
            )),
        },
    }
}

fn parse_short(token: &str, argv: &[String], index: &mut usize) -> Result<Matched, String> {
    let body = &token[1..];
    for (pos, letter) in body.char_indices() {
        let opt = match letter {
            'h' => Opt::Help,
            'f' => Opt::File,
            'o' => Opt::Out,
            _ => return Ok(Matched::Unknown(token.to_string())),
        };
        if !opt.takes_value() {
            return Ok(Matched::Option(opt, None));
        }
        let rest = &body[pos + letter.len_utf8()..];
        let value = if rest.is_empty() {
            match next_value(argv, index) {
                Some(value) => value,
                None => {
                    return Err(format!(
                        "argument {}: expected one argument",
                        opt.display()
                    ))
                }
            }
        } else {
            rest.strip_prefix('=').unwrap_or(rest).to_string()
        };
        return Ok(Matched::Option(opt, Some(value)));
    }
    Ok(Matched::Unknown(token.to_string()))
}

fn next_value(argv: &[String], index: &mut usize) -> Option<String> {
    let token = argv.get(*index)?;
    if token == "-" || !token.starts_with('-') || is_negative_number(token) {
        *index += 1;
        Some(token.clone())
    } else {
        None
    }
}

fn apply(state: &mut State, opt: Opt, value: Option<String>) -> Option<Outcome> {
    match opt {
        Opt::Help => Some(Outcome::Help),
        Opt::Version => Some(Outcome::Version),
        Opt::Html => {
            state.html = true;
            None
        }
        Opt::Term => {
            state.term = true;
            None
        }
        Opt::File => {
            state.file = Some(value.unwrap_or_default());
            None
        }
        Opt::Out => {
            state.out = value.unwrap_or_default();
            state.out_given = true;
            None
        }
        Opt::Title => {
            state.title = value.unwrap_or_default();
            state.title_given = true;
            None
        }
        Opt::Scale => match int_value(value, Opt::Scale) {
            Ok(number) => {
                state.scale = number;
                state.scale_given = true;
                None
            }
            Err(message) => Some(Outcome::Usage(message)),
        },
        Opt::Border => match int_value(value, Opt::Border) {
            Ok(number) => {
                state.border = number;
                None
            }
            Err(message) => Some(Outcome::Usage(message)),
        },
    }
}

fn int_value(value: Option<String>, opt: Opt) -> Result<i64, String> {
    let text = value.unwrap_or_default();
    match text.trim().parse::<i64>() {
        Ok(number) => Ok(number),
        Err(_) => Err(format!(
            "argument {}: invalid int value: '{}'",
            opt.display(),
            text
        )),
    }
}

fn finish(state: State) -> Outcome {
    if !state.extras.is_empty() {
        return Outcome::Usage(format!(
            "unrecognized arguments: {}",
            state.extras.join(" ")
        ));
    }
    if !(1..=100).contains(&state.scale) {
        return Outcome::Usage("--scale must be 1-100".to_string());
    }
    if !(0..=16).contains(&state.border) {
        return Outcome::Usage("--border must be 0-16".to_string());
    }
    if state.file.is_some() && state.text.is_some() {
        return Outcome::Usage("give text or --file, not both".to_string());
    }
    if state.file.is_none() && state.text.is_none() {
        return Outcome::Usage("provide text or --file".to_string());
    }
    if state.term {
        if state.out_given {
            return Outcome::Usage("--term writes no file; drop --out".to_string());
        }
        if state.html {
            return Outcome::Usage("--term writes no file; drop --html".to_string());
        }
        if state.title_given {
            return Outcome::Usage("--term writes no file; drop --title".to_string());
        }
        if state.scale_given {
            return Outcome::Usage("--term writes no file; drop --scale".to_string());
        }
    }
    Outcome::Run(Args {
        text: state.text,
        file: state.file,
        out: state.out,
        scale: state.scale as u32,
        border: state.border as u32,
        html: state.html,
        title: state.title,
        term: state.term,
    })
}

fn is_negative_number(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('-') else {
        return false;
    };
    let mut dot = false;
    let mut digits = 0;
    for c in rest.chars() {
        if c == '.' {
            if dot {
                return false;
            }
            dot = true;
        } else if c.is_ascii_digit() {
            digits += 1;
        } else {
            return false;
        }
    }
    digits > 0
}

/// Wrapped usage line, as argparse prints it.
pub fn usage(prog: &str) -> String {
    let prefix = format!("usage: {} ", prog);
    let pad = " ".repeat(prefix.len());
    let mut lines: Vec<String> = Vec::new();
    let mut line = prefix.clone();
    for token in USAGE_TAIL.split(' ') {
        if line.len() + token.len() > WIDTH && line != prefix {
            lines.push(line.trim_end().to_string());
            line = pad.clone();
        }
        line.push_str(token);
        line.push(' ');
    }
    lines.push(line.trim_end().to_string());
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

/// The help text. --version is omitted so the text stays equal to the frozen fixture.
pub fn help(prog: &str) -> String {
    let mut text = usage(prog);
    text.push('\n');
    text.push_str("Generate an offline QR code PNG or print it to the terminal.\n\n");
    text.push_str("positional arguments:\n");
    text.push_str("  text             text to encode; omit when using --file\n\n");
    text.push_str("options:\n");
    text.push_str("  -h, --help       show this help message and exit\n");
    text.push_str("  -f, --file FILE  read the text from a file, stripped\n");
    text.push_str("  -o, --out OUT    output PNG path\n");
    text.push_str("  --scale SCALE    pixels per module 1-100 (default 10)\n");
    text.push_str("  --border BORDER  quiet zone in modules 0-16 (default 4)\n");
    text.push_str("  --html           write a viewer page next to the PNG\n");
    text.push_str("  --title TITLE    viewer page title\n");
    text.push_str("  --term           print the QR code to the terminal, write no file\n");
    text
}

/// A usage error block: usage line, then prog: error: message.
pub fn error(prog: &str, message: &str) -> String {
    format!("{}{}: error: {}\n", usage(prog), prog, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn usage_message(items: &[&str]) -> String {
        match parse(&argv(items)) {
            Outcome::Usage(message) => message,
            other => panic!("expected a usage error, got {:?}", other),
        }
    }

    #[test]
    fn default_run_has_the_documented_defaults() {
        match parse(&argv(&["x"])) {
            Outcome::Run(args) => {
                assert_eq!(args.text.as_deref(), Some("x"));
                assert_eq!(args.out, "qr.png");
                assert_eq!(args.scale, 10);
                assert_eq!(args.border, 4);
                assert!(!args.html);
                assert_eq!(args.title, "qr code");
                assert!(!args.term);
            }
            other => panic!("expected a run, got {:?}", other),
        }
    }

    #[test]
    fn long_prefixes_and_equals_forms_work() {
        match parse(&argv(&["x", "--sc=12", "--bo", "2", "--out=a.png"])) {
            Outcome::Run(args) => {
                assert_eq!(args.scale, 12);
                assert_eq!(args.border, 2);
                assert_eq!(args.out, "a.png");
            }
            other => panic!("expected a run, got {:?}", other),
        }
    }

    #[test]
    fn help_wins_inside_a_short_cluster() {
        assert_eq!(parse(&argv(&["-h"])), Outcome::Help);
        assert_eq!(parse(&argv(&["-hx"])), Outcome::Help);
    }

    #[test]
    fn version_is_a_hidden_flag() {
        assert_eq!(parse(&argv(&["--version"])), Outcome::Version);
        assert_eq!(parse(&argv(&["--vers"])), Outcome::Version);
    }

    #[test]
    fn ambiguous_prefix_lists_matches_in_order() {
        assert_eq!(
            usage_message(&["x", "--h"]),
            "ambiguous option: --h could match --help, --html"
        );
    }

    #[test]
    fn unknown_options_are_deferred_to_extras() {
        assert_eq!(usage_message(&["x", "--bogus"]), "unrecognized arguments: --bogus");
        assert_eq!(usage_message(&["x", "y"]), "unrecognized arguments: y");
        assert_eq!(usage_message(&["-xh"]), "unrecognized arguments: -xh");
    }

    #[test]
    fn option_looking_values_are_refused() {
        assert_eq!(
            usage_message(&["x", "--out", "--html"]),
            "argument -o/--out: expected one argument"
        );
        assert_eq!(
            usage_message(&["x", "--scale"]),
            "argument --scale: expected one argument"
        );
    }

    #[test]
    fn bad_ints_and_ranges_are_reported() {
        assert_eq!(
            usage_message(&["x", "--scale", "abc"]),
            "argument --scale: invalid int value: 'abc'"
        );
        assert_eq!(usage_message(&["x", "--scale", "0"]), "--scale must be 1-100");
        assert_eq!(usage_message(&["x", "--border", "-1"]), "--border must be 0-16");
    }

    #[test]
    fn separator_and_dash_are_positional() {
        match parse(&argv(&["--", "--scale"])) {
            Outcome::Run(args) => assert_eq!(args.text.as_deref(), Some("--scale")),
            other => panic!("expected a run, got {:?}", other),
        }
        match parse(&argv(&["-"])) {
            Outcome::Run(args) => assert_eq!(args.text.as_deref(), Some("-")),
            other => panic!("expected a run, got {:?}", other),
        }
    }

    #[test]
    fn sources_are_checked_like_the_python_line() {
        assert_eq!(usage_message(&[]), "provide text or --file");
        assert_eq!(
            usage_message(&["x", "-f", "secret.txt"]),
            "give text or --file, not both"
        );
        assert_eq!(usage_message(&["x", "--border", "17"]), "--border must be 0-16");
    }

    #[test]
    fn help_flat_matches_the_frozen_fixture() {
        let flat = help("make-qr.py").split_whitespace().collect::<Vec<_>>().join(" ");
        let want = "usage: make-qr.py [-h] [-f FILE] [-o OUT] [--scale SCALE] [--border BORDER] [--html] [--title TITLE] [--term] [text] Generate an offline QR code PNG or print it to the terminal. positional arguments: text text to encode; omit when using --file options: -h, --help show this help message and exit -f, --file FILE read the text from a file, stripped -o, --out OUT output PNG path --scale SCALE pixels per module 1-100 (default 10) --border BORDER quiet zone in modules 0-16 (default 4) --html write a viewer page next to the PNG --title TITLE viewer page title --term print the QR code to the terminal, write no file";
        assert_eq!(flat, want);
    }

    #[test]
    fn term_is_a_flag_with_a_quiet_zone() {
        match parse(&argv(&["x", "--term", "--border", "2"])) {
            Outcome::Run(args) => {
                assert!(args.term);
                assert_eq!(args.border, 2);
                assert_eq!(args.out, "qr.png");
            }
            other => panic!("expected a run, got {:?}", other),
        }
        match parse(&argv(&["x", "--te"])) {
            Outcome::Run(args) => assert!(args.term),
            other => panic!("expected a run, got {:?}", other),
        }
    }

    #[test]
    fn term_refuses_every_output_option() {
        assert_eq!(
            usage_message(&["x", "--term", "-o", "a.png"]),
            "--term writes no file; drop --out"
        );
        assert_eq!(
            usage_message(&["x", "--term", "--html"]),
            "--term writes no file; drop --html"
        );
        assert_eq!(
            usage_message(&["x", "--term", "--title", "t"]),
            "--term writes no file; drop --title"
        );
        assert_eq!(
            usage_message(&["x", "--term", "--scale", "12"]),
            "--term writes no file; drop --scale"
        );
        assert_eq!(
            usage_message(&["x", "--term", "--out=a.png"]),
            "--term writes no file; drop --out"
        );
    }

    #[test]
    fn term_still_needs_a_source() {
        assert_eq!(usage_message(&["--term"]), "provide text or --file");
    }

    #[test]
    fn title_and_term_share_the_t_prefix() {
        assert_eq!(
            usage_message(&["x", "--t"]),
            "ambiguous option: --t could match --title, --term"
        );
    }

    #[test]
    fn usage_errors_carry_the_usage_line() {
        let text = error("qr", "empty input");
        assert!(text.starts_with("usage: qr "));
        assert!(text.ends_with("qr: error: empty input\n"));
    }
}
