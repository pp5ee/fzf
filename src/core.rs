use crate::cache::ChunkCache;
use crate::matcher::Matcher;
use crate::options::{Algo, Case, Layout, Options, Scheme, BorderStyle};
use crate::pattern::{Pattern, PatternOptions};
use crate::reader::{Reader, EVT_READ_FIN, EVT_READ_NEW};
use crate::util::eventbox::{EventBox, EventType, EventValue};
use crate::util::Executor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::{Result, anyhow};

pub const EXIT_OK: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_NO_MATCH: i32 = 1;
pub const EXIT_SIGINT: i32 = 130;

pub struct Core {
    options: Options,
    reader: Option<Reader>,
    matcher: Option<Matcher>,
    event_box: EventBox,
    running: Arc<AtomicBool>,
}

impl Core {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            reader: None,
            matcher: None,
            event_box: EventBox::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&mut self) -> Result<i32> {
        // Initialize algo scheme
        let scheme_str = match self.options.scheme {
            Scheme::Path => "path",
            Scheme::History => "history",
            _ => "default",
        };
        crate::algo::Scheme::init(scheme_str);

        // Handle shell integration output
        if self.options.bash {
            print_shell_integration("bash")?;
            return Ok(EXIT_OK);
        }
        if self.options.zsh {
            print_shell_integration("zsh")?;
            return Ok(EXIT_OK);
        }
        if self.options.fish {
            print_shell_integration("fish")?;
            return Ok(EXIT_OK);
        }

        // Handle version/help
        if self.options.version {
            println!("0.72.0");
            return Ok(EXIT_OK);
        }
        if self.options.help {
            print_help();
            return Ok(EXIT_OK);
        }

        // Filter mode (non-interactive)
        if let Some(ref filter) = self.options.filter {
            return self.run_filter_mode(filter);
        }

        // Interactive mode
        self.run_interactive_mode()
    }

    fn run_filter_mode(&self, filter: &str) -> Result<i32> {
        let executor = Executor::new();
        let reader = Reader::new(
            EventBox::new(),
            executor,
            self.options.read0,
        );

        // Read all input
        reader.read_stdin()?;

        let chunk_list = reader.chunk_list();
        let list = chunk_list.lock().map_err(|_| anyhow!("Lock poisoned"))?;

        // Create pattern
        let pattern_opts = PatternOptions {
            fuzzy: self.options.fuzzy,
            fuzzy_algo: match self.options.algo {
                Algo::V1 => crate::algo::Algo::V1,
                Algo::V2 => crate::algo::Algo::V2,
            },
            extended: self.options.extended,
            case_sensitive: self.options.case_sensitive == Some(true),
            normalize: self.options.normalize,
            forward: true,
            with_pos: false,
        };
        let pattern = Arc::new(Pattern::new(filter, &pattern_opts));

        // Match items
        let mut results = Vec::new();
        let delim = if self.options.print0 { '\0' } else { '\n' };

        for chunk in list.chunks() {
            for item in &chunk.items {
                let text = item.to_vec();
                if let Some(_matched) = pattern.match_text(&text) {
                    let output = item.as_str();
                    if !output.is_empty() {
                        results.push(output.to_string());
                    }
                }
            }
        }

        drop(list);

        // Output results
        if results.is_empty() {
            if self.options.exit_0 {
                return Ok(EXIT_OK);
            }
            return Ok(EXIT_NO_MATCH);
        }

        // Apply tac if needed
        if self.options.tac {
            results.reverse();
        }

        // Output
        for (i, result) in results.iter().enumerate() {
            print!("{}", result);
            if i < results.len() - 1 || !result.ends_with(delim) {
                print!("{}", delim);
            }
        }

        Ok(EXIT_OK)
    }

    fn run_interactive_mode(&self) -> Result<i32> {
        // For now, fall back to filter mode with empty filter
        // Full terminal UI would require significant additional implementation
        self.run_filter_mode("")
    }
}

pub fn run(options: Options) -> Result<i32> {
    let mut core = Core::new(options);
    core.run()
}

fn print_shell_integration(shell: &str) -> Result<()> {
    let script = match shell {
        "bash" => include_str!("../shell/key-bindings.bash"),
        "zsh" => include_str!("../shell/key-bindings.zsh"),
        "fish" => include_str!("../shell/key-bindings.fish"),
        _ => return Err(anyhow!("Unsupported shell: {}", shell)),
    };
    println!("{}", script);
    Ok(())
}

fn print_help() {
    println!("fzf - a command-line fuzzy finder");
    println!();
    println!("Usage: fzf [options]");
    println!();
    println!("  SEARCH");
    println!("    -e, --exact              Enable exact-match");
    println!("    -i, --ignore-case        Case-insensitive match");
    println!("    +i, --no-ignore-case     Case-sensitive match");
    println!("        --smart-case         Smart-case match (default)");
    println!("    --literal                Do not normalize latin script letters");
    println!("    -n, --nth=N[,..]         Comma-separated list of field index expressions");
    println!("    --with-nth=N[,..]        Transform the presentation of each line");
    println!("    --accept-nth=N[,..]      Define which fields to print on accept");
    println!("    -d, --delimiter=STR      Field delimiter regex");
    println!("    +s, --no-sort            Do not sort the result");
    println!("    --tac                    Reverse the order of the input");
    println!("    --disabled               Do not perform search");
    println!();
    println!("  INPUT/OUTPUT");
    println!("    --read0                  Read input delimited by ASCII NUL characters");
    println!("    --print0                 Print output delimited by ASCII NUL characters");
    println!("    --ansi                   Enable processing of ANSI color codes");
    println!();
    println!("  DISPLAY");
    println!("    --height=HEIGHT          Display fzf window with the given height");
    println!("    --layout=LAYOUT          Choose layout: default|reverse|reverse-list");
    println!("    --border[=STYLE]         Draw border around the finder");
    println!("    --prompt=STR             Input prompt (default: '> ' )");
    println!();
    println!("  For more information, see: https://github.com/junegunn/fzf");
}
