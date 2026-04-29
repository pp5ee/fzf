use clap::{Arg, ArgAction, Command};

#[derive(Debug, Clone)]
pub struct Options {
    pub filter: Option<String>,
    pub exact: bool,
    pub extended: bool,
    pub fuzzy: bool,
    pub case_sensitive: Option<bool>,
    pub smart_case: bool,
    pub normalize: bool,
    pub literal: bool,
    pub nth: Option<String>,
    pub with_nth: Option<String>,
    pub accept_nth: Option<String>,
    pub delimiter: Option<String>,
    pub sort: usize,
    pub tac: bool,
    pub ansi: bool,
    pub read0: bool,
    pub print0: bool,
    pub sync: bool,
    pub multi: Option<usize>,
    pub preview: Option<String>,
    pub preview_window: Option<String>,
    pub height: Option<String>,
    pub min_height: Option<usize>,
    pub layout: Layout,
    pub border: BorderStyle,
    pub prompt: String,
    pub header: Option<String>,
    pub header_lines: usize,
    pub footer: Option<String>,
    pub info: InfoStyle,
    pub pointer: String,
    pub marker: String,
    pub tabstop: usize,
    pub history: Option<String>,
    pub history_size: usize,
    pub query: Option<String>,
    pub select_1: bool,
    pub exit_0: bool,
    pub bind: Vec<String>,
    pub color: Option<String>,
    pub no_color: bool,
    pub no_mouse: bool,
    pub no_hscroll: bool,
    pub keep_right: bool,
    pub cycle: bool,
    pub reverse: bool,
    pub disabled: bool,
    pub strip_xterm: bool,
    pub exit_130: bool,
    pub scroll_off: usize,
    pub hscroll_off: usize,
    pub jump_labels: Option<String>,
    pub margin: Option<String>,
    pub padding: Option<String>,
    pub ellipsis: String,
    pub scheme: Scheme,
    pub algo: Algo,
    pub case: Case,
    pub tiebreak: Vec<Tiebreak>,
    pub expect: Vec<String>,
    pub no_sort: bool,
    pub track: bool,
    pub tail: Option<usize>,
    pub listen: Option<String>,
    pub version: bool,
    pub help: bool,
    pub bash: bool,
    pub zsh: bool,
    pub fish: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Default,
    Reverse,
    ReverseList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Rounded,
    Sharp,
    Bold,
    Block,
    ThinBlock,
    Double,
    Horizontal,
    Vertical,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoStyle {
    Default,
    Right,
    Hidden,
    Inline,
    InlineRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Default,
    Path,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algo {
    V1,
    V2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Smart,
    Ignore,
    Respect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tiebreak {
    Length,
    Chunk,
    Pathname,
    Begin,
    End,
    Index,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            filter: None,
            exact: false,
            extended: true,
            fuzzy: true,
            case_sensitive: None,
            smart_case: true,
            normalize: true,
            literal: false,
            nth: None,
            with_nth: None,
            accept_nth: None,
            delimiter: None,
            sort: 1000,
            tac: false,
            ansi: false,
            read0: false,
            print0: false,
            sync: false,
            multi: None,
            preview: None,
            preview_window: None,
            height: None,
            min_height: Some(10),
            layout: Layout::Default,
            border: BorderStyle::Rounded,
            prompt: "> ".to_string(),
            header: None,
            header_lines: 0,
            footer: None,
            info: InfoStyle::Default,
            pointer: "▌".to_string(),
            marker: "┃".to_string(),
            tabstop: 8,
            history: None,
            history_size: 1000,
            query: None,
            select_1: false,
            exit_0: false,
            bind: Vec::new(),
            color: None,
            no_color: false,
            no_mouse: false,
            no_hscroll: false,
            keep_right: false,
            cycle: false,
            reverse: false,
            disabled: false,
            strip_xterm: false,
            exit_130: false,
            scroll_off: 0,
            hscroll_off: 10,
            jump_labels: None,
            margin: None,
            padding: None,
            ellipsis: "··".to_string(),
            scheme: Scheme::Default,
            algo: Algo::V2,
            case: Case::Smart,
            tiebreak: vec![Tiebreak::Length],
            expect: Vec::new(),
            no_sort: false,
            track: false,
            tail: None,
            listen: None,
            version: false,
            help: false,
            bash: false,
            zsh: false,
            fish: false,
        }
    }
}

pub fn parse_options(args: &[String]) -> anyhow::Result<Options> {
    let mut opts = Options::default();

    let cmd = Command::new("fzf")
        .about("A command-line fuzzy finder")
        .arg(Arg::new("filter")
            .long("filter")
            .short('f')
            .value_name("STR")
            .help("Filter mode. Do not start interactive finder."))
        .arg(Arg::new("exact")
            .long("exact")
            .short('e')
            .action(ArgAction::SetTrue)
            .help("Enable exact-match"))
        .arg(Arg::new("extended")
            .long("extended")
            .short('x')
            .action(ArgAction::SetTrue)
            .help("Enable extended-search mode"))
        .arg(Arg::new("no-extended")
            .long("no-extended")
            .action(ArgAction::SetTrue)
            .help("Disable extended-search mode"))
        .arg(Arg::new("ignore-case")
            .long("ignore-case")
            .short('i')
            .action(ArgAction::SetTrue)
            .help("Case-insensitive match"))
        .arg(Arg::new("no-ignore-case")
            .long("no-ignore-case")
            .action(ArgAction::SetTrue)
            .help("Case-sensitive match"))
        .arg(Arg::new("smart-case")
            .long("smart-case")
            .action(ArgAction::SetTrue)
            .help("Smart-case match (default)"))
        .arg(Arg::new("literal")
            .long("literal")
            .action(ArgAction::SetTrue)
            .help("Do not normalize latin script letters"))
        .arg(Arg::new("nth")
            .long("nth")
            .short('n')
            .value_name("N[,..]")
            .help("Comma-separated list of field index expressions"))
        .arg(Arg::new("with-nth")
            .long("with-nth")
            .value_name("N[,..]")
            .help("Transform the presentation of each line using field index expressions"))
        .arg(Arg::new("accept-nth")
            .long("accept-nth")
            .value_name("N[,..]")
            .help("Define which fields to print on accept"))
        .arg(Arg::new("delimiter")
            .long("delimiter")
            .short('d')
            .value_name("STR")
            .help("Field delimiter regex (default: AWK-style)"))
        .arg(Arg::new("no-sort")
            .long("no-sort")
            .short('+')
            .action(ArgAction::SetTrue)
            .help("Do not sort the result"))
        .arg(Arg::new("tac")
            .long("tac")
            .action(ArgAction::SetTrue)
            .help("Reverse the order of the input"))
        .arg(Arg::new("read0")
            .long("read0")
            .action(ArgAction::SetTrue)
            .help("Read input delimited by ASCII NUL characters"))
        .arg(Arg::new("print0")
            .long("print0")
            .action(ArgAction::SetTrue)
            .help("Print output delimited by ASCII NUL characters"))
        .arg(Arg::new("ansi")
            .long("ansi")
            .action(ArgAction::SetTrue)
            .help("Enable processing of ANSI color codes"))
        .arg(Arg::new("sync")
            .long("sync")
            .action(ArgAction::SetTrue)
            .help("Synchronous search for multi-staged filtering"))
        .arg(Arg::new("multi")
            .long("multi")
            .short('m')
            .value_name("MAX")
            .num_args(0..=1)
            .default_missing_value("0")
            .action(ArgAction::Set)
            .help("Enable multi-select with tab/shift-tab"))
        .arg(Arg::new("preview")
            .long("preview")
            .value_name("COMMAND")
            .help("Command to preview highlighted line"))
        .arg(Arg::new("preview-window")
            .long("preview-window")
            .value_name("OPT")
            .help("Preview window layout"))
        .arg(Arg::new("height")
            .long("height")
            .value_name("HEIGHT")
            .help("Display fzf window with the given height"))
        .arg(Arg::new("min-height")
            .long("min-height")
            .value_name("N")
            .help("Minimum height when --height is given"))
        .arg(Arg::new("layout")
            .long("layout")
            .value_name("LAYOUT")
            .help("Choose layout: default|reverse|reverse-list"))
        .arg(Arg::new("border")
            .long("border")
            .value_name("STYLE")
            .action(ArgAction::Set)
            .help("Draw border around the finder"))
        .arg(Arg::new("prompt")
            .long("prompt")
            .value_name("STR")
            .help("Input prompt"))
        .arg(Arg::new("header")
            .long("header")
            .value_name("STR")
            .help("String to print as header"))
        .arg(Arg::new("header-lines")
            .long("header-lines")
            .value_name("N")
            .help("Number of header lines from input"))
        .arg(Arg::new("footer")
            .long("footer")
            .value_name("STR")
            .help("String to print as footer"))
        .arg(Arg::new("info")
            .long("info")
            .value_name("STYLE")
            .help("Finder info style"))
        .arg(Arg::new("pointer")
            .long("pointer")
            .value_name("STR")
            .help("Pointer to the current line"))
        .arg(Arg::new("marker")
            .long("marker")
            .value_name("STR")
            .help("Multi-select marker"))
        .arg(Arg::new("tabstop")
            .long("tabstop")
            .value_name("SPACES")
            .help("Number of spaces for a tab character"))
        .arg(Arg::new("history")
            .long("history")
            .value_name("FILE")
            .help("History file"))
        .arg(Arg::new("query")
            .long("query")
            .short('q')
            .value_name("STR")
            .help("Start with the given query"))
        .arg(Arg::new("select-1")
            .long("select-1")
            .action(ArgAction::SetTrue)
            .help("Automatically select the only match"))
        .arg(Arg::new("exit-0")
            .long("exit-0")
            .action(ArgAction::SetTrue)
            .help("Exit immediately when there's no match"))
        .arg(Arg::new("bind")
            .long("bind")
            .value_name("KEYS:ACTION")
            .action(ArgAction::Append)
            .help("Custom key bindings"))
        .arg(Arg::new("color")
            .long("color")
            .value_name("COLSPEC")
            .help("Color scheme"))
        .arg(Arg::new("no-color")
            .long("no-color")
            .action(ArgAction::SetTrue)
            .help("Disable colors"))
        .arg(Arg::new("no-mouse")
            .long("no-mouse")
            .action(ArgAction::SetTrue)
            .help("Disable mouse"))
        .arg(Arg::new("no-hscroll")
            .long("no-hscroll")
            .action(ArgAction::SetTrue)
            .help("Disable horizontal scroll"))
        .arg(Arg::new("keep-right")
            .long("keep-right")
            .action(ArgAction::SetTrue)
            .help("Keep the right end of the line visible"))
        .arg(Arg::new("cycle")
            .long("cycle")
            .action(ArgAction::SetTrue)
            .help("Enable cyclic scroll"))
        .arg(Arg::new("reverse")
            .long("reverse")
            .action(ArgAction::SetTrue)
            .help("Reverse orientation"))
        .arg(Arg::new("disabled")
            .long("disabled")
            .action(ArgAction::SetTrue)
            .help("Do not perform search"))
        .arg(Arg::new("scheme")
            .long("scheme")
            .value_name("SCHEME")
            .help("Scoring scheme: default|path|history"))
        .arg(Arg::new("algo")
            .long("algo")
            .value_name("ALGO")
            .help("Fuzzy matching algorithm: v1|v2"))
        .arg(Arg::new("tiebreak")
            .long("tiebreak")
            .value_name("CRI[,..]")
            .help("Comma-separated list of sort criteria"))
        .arg(Arg::new("expect")
            .long("expect")
            .value_name("KEYS")
            .help("Comma-separated list of keys to complete fzf"))
        .arg(Arg::new("track")
            .long("track")
            .action(ArgAction::SetTrue)
            .help("Track the current selection"))
        .arg(Arg::new("tail")
            .long("tail")
            .value_name("NUM")
            .help("Maximum number of items to keep in memory"))
        .arg(Arg::new("listen")
            .long("listen")
            .value_name("ADDR")
            .num_args(0..=1)
            .help("Start HTTP server for remote control"))
        .arg(Arg::new("version-flag")
            .long("version")
            .action(ArgAction::SetTrue)
            .help("Show version"))
        .arg(Arg::new("help-flag")
            .long("help")
            .action(ArgAction::SetTrue)
            .help("Show help"))
        .arg(Arg::new("bash")
            .long("bash")
            .action(ArgAction::SetTrue)
            .help("Output bash integration scripts"))
        .arg(Arg::new("zsh")
            .long("zsh")
            .action(ArgAction::SetTrue)
            .help("Output zsh integration scripts"))
        .arg(Arg::new("fish")
            .long("fish")
            .action(ArgAction::SetTrue)
            .help("Output fish integration scripts"));

    let matches = cmd.try_get_matches_from(args)?;

    // Parse boolean flags
    opts.filter = matches.get_one::<String>("filter").cloned();
    opts.exact = matches.get_flag("exact");
    opts.extended = !matches.get_flag("no-extended");
    opts.fuzzy = !matches.get_flag("exact");

    if matches.get_flag("ignore-case") {
        opts.case_sensitive = Some(false);
        opts.smart_case = false;
    } else if matches.get_flag("no-ignore-case") {
        opts.case_sensitive = Some(true);
        opts.smart_case = false;
    }

    opts.literal = matches.get_flag("literal");
    opts.normalize = !opts.literal;

    opts.nth = matches.get_one::<String>("nth").cloned();
    opts.with_nth = matches.get_one::<String>("with_nth").cloned();
    opts.accept_nth = matches.get_one::<String>("accept_nth").cloned();
    opts.delimiter = matches.get_one::<String>("delimiter").cloned();

    opts.no_sort = matches.get_flag("no-sort");
    opts.tac = matches.get_flag("tac");
    opts.read0 = matches.get_flag("read0");
    opts.print0 = matches.get_flag("print0");
    opts.ansi = matches.get_flag("ansi");
    opts.sync = matches.get_flag("sync");

    if let Some(multi) = matches.get_one::<String>("multi") {
        opts.multi = if multi == "0" {
            None
        } else {
            multi.parse().ok()
        };
    }

    opts.preview = matches.get_one::<String>("preview").cloned();
    opts.preview_window = matches.get_one::<String>("preview_window").cloned();
    opts.height = matches.get_one::<String>("height").cloned();
    opts.min_height = matches.get_one::<String>("min_height")
        .and_then(|s| s.parse().ok());

    if let Some(layout) = matches.get_one::<String>("layout") {
        opts.layout = match layout.as_str() {
            "reverse" => Layout::Reverse,
            "reverse-list" => Layout::ReverseList,
            _ => Layout::Default,
        };
    }

    if let Some(border) = matches.get_one::<String>("border") {
        opts.border = match border.as_str() {
            "none" => BorderStyle::None,
            "sharp" => BorderStyle::Sharp,
            "bold" => BorderStyle::Bold,
            "block" => BorderStyle::Block,
            "thinblock" => BorderStyle::ThinBlock,
            "double" => BorderStyle::Double,
            "horizontal" => BorderStyle::Horizontal,
            "vertical" => BorderStyle::Vertical,
            "top" => BorderStyle::Top,
            "bottom" => BorderStyle::Bottom,
            "left" => BorderStyle::Left,
            "right" => BorderStyle::Right,
            _ => BorderStyle::Rounded,
        };
    } else if matches.get_flag("border") {
        opts.border = BorderStyle::Rounded;
    }

    if let Some(prompt) = matches.get_one::<String>("prompt") {
        opts.prompt = prompt.clone();
    }

    opts.header = matches.get_one::<String>("header").cloned();
    opts.header_lines = matches.get_one::<String>("header_lines")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    opts.footer = matches.get_one::<String>("footer").cloned();

    if let Some(info) = matches.get_one::<String>("info") {
        opts.info = match info.as_str() {
            "right" => InfoStyle::Right,
            "hidden" => InfoStyle::Hidden,
            "inline" => InfoStyle::Inline,
            "inline-right" => InfoStyle::InlineRight,
            _ => InfoStyle::Default,
        };
    }

    if let Some(pointer) = matches.get_one::<String>("pointer") {
        opts.pointer = pointer.clone();
    }

    if let Some(marker) = matches.get_one::<String>("marker") {
        opts.marker = marker.clone();
    }

    opts.tabstop = matches.get_one::<String>("tabstop")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);

    opts.history = matches.get_one::<String>("history").cloned();
    opts.query = matches.get_one::<String>("query").cloned();
    opts.select_1 = matches.get_flag("select-1");
    opts.exit_0 = matches.get_flag("exit-0");
    opts.color = matches.get_one::<String>("color").cloned();
    opts.no_color = matches.get_flag("no-color");
    opts.no_mouse = matches.get_flag("no-mouse");
    opts.no_hscroll = matches.get_flag("no-hscroll");
    opts.keep_right = matches.get_flag("keep-right");
    opts.cycle = matches.get_flag("cycle");
    opts.reverse = matches.get_flag("reverse");
    opts.disabled = matches.get_flag("disabled");

    if let Some(scheme) = matches.get_one::<String>("scheme") {
        opts.scheme = match scheme.as_str() {
            "path" => Scheme::Path,
            "history" => Scheme::History,
            _ => Scheme::Default,
        };
    }

    if let Some(algo) = matches.get_one::<String>("algo") {
        opts.algo = match algo.as_str() {
            "v1" => Algo::V1,
            _ => Algo::V2,
        };
    }

    if let Some(tiebreak) = matches.get_one::<String>("tiebreak") {
        opts.tiebreak = tiebreak
            .split(',')
            .filter_map(|s| match s.trim() {
                "length" => Some(Tiebreak::Length),
                "chunk" => Some(Tiebreak::Chunk),
                "pathname" => Some(Tiebreak::Pathname),
                "begin" => Some(Tiebreak::Begin),
                "end" => Some(Tiebreak::End),
                "index" => Some(Tiebreak::Index),
                _ => None,
            })
            .collect();
    }

    opts.expect = matches.get_one::<String>("expect")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    opts.track = matches.get_flag("track");
    opts.tail = matches.get_one::<String>("tail")
        .and_then(|s| s.parse().ok());
    opts.listen = matches.get_one::<String>("listen").cloned();

    opts.version = matches.get_flag("version-flag");
    opts.help = matches.get_flag("help-flag");
    opts.bash = matches.get_flag("bash");
    opts.zsh = matches.get_flag("zsh");
    opts.fish = matches.get_flag("fish");

    if let Some(binds) = matches.get_many::<String>("bind") {
        opts.bind = binds.cloned().collect();
    }

    // Validate options
    if opts.no_color {
        opts.color = Some("bw".to_string());
    }

    Ok(opts)
}
