//! The `uci` module contains the definitions that represent UCI protocol messages.
//!
//! Usually, these messages will be obtained by calling the `parse` method of the `parser` module, but you can always
//! construct them in code and then print them to the standard output to communicate with the GUI.

use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
#[cfg(not(feature = "chess"))]
use std::str::FromStr;

#[cfg(feature = "chess")]
use chess::ChessMove;
use chrono::Duration;
use pest::error::Error as PestError;

#[cfg(feature = "specta")]
use specta::Type;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::parser::Rule;

/// Specifies whether a message is engine- or GUI-bound.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CommunicationDirection {
    /// An engine-bound message.
    GuiToEngine,

    /// A GUI-bound message.
    EngineToGui,
}

pub trait UciSerializable: Display {
    fn uci_serialize(&self) -> String;
}

/// An enumeration type containing representations for all messages supported by the UCI protocol.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciMessage {
    /// The `uci` engine-bound message.
    Uci,

    /// The `debug` engine-bound message. Its internal property specifies whether debug mode should be enabled (`true`),
    /// or disabled (`false`).
    Debug(bool),

    /// The `isready` engine-bound message.
    IsReady,

    /// The `register` engine-bound message.
    Register {
        /// The `register later` engine-bound message.
        later: bool,

        /// The name part of the `register <code> <name>` engine-bound message.
        name: Option<String>,

        /// The code part of the `register <code> <name>` engine-bound message.
        code: Option<String>,
    },

    /// The `position` engine-bound message.
    Position {
        /// If `true`, it denotes the starting chess position. Generally, if this property is `true`, then the value of
        /// the `fen` property will be `None`.
        startpos: bool,

        /// The [FEN format](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) representation of a chess
        /// position.
        fen: Option<UciFen>,

        /// A list of moves to apply to the position.
        #[cfg(not(feature = "chess"))]
        moves: Vec<UciMove>,

        /// A list of moves to apply to the position.
        #[cfg(feature = "chess")]
        moves: Vec<ChessMove>,
    },

    /// The `setoption` engine-bound message.
    SetOption {
        /// The name of the option to set.
        name: String,

        /// The value of the option to set. If the option has no value, this should be `None`.
        value: Option<String>,
    },

    /// The `ucinewgame` engine-bound message.
    UciNewGame,

    /// The `stop` engine-bound message.
    Stop,

    /// The `ponderhit` engine-bound message.
    PonderHit,

    /// The `quit` engine-bound message.
    Quit,

    /// The `go` engine-bound message.
    Go {
        /// Time-control-related `go` parameters (sub-commands).
        time_control: Option<UciTimeControl>,

        /// Search-related `go` parameters (sub-commands).
        search_control: Option<UciSearchControl>,
    },

    // From this point on we have client-bound messages
    /// The `id` GUI-bound message.
    Id {
        /// The name of the engine, possibly including the version.
        name: Option<String>,

        /// The name of the author of the engine.
        author: Option<String>,
    },

    /// The `uciok` GUI-bound message.
    UciOk,

    /// The `readyok` GUI-bound message.
    ReadyOk,

    /// The `bestmove` GUI-bound message.
    BestMove {
        /// The move the engine thinks is the best one in the position.
        #[cfg(not(feature = "chess"))]
        best_move: UciMove,

        /// The move the engine thinks is the best one in the position.
        #[cfg(feature = "chess")]
        best_move: ChessMove,

        /// The move the engine would like to ponder on.
        #[cfg(not(feature = "chess"))]
        ponder: Option<UciMove>,

        /// The move the engine would like to ponder on.
        #[cfg(feature = "chess")]
        ponder: Option<ChessMove>,
    },

    /// The `copyprotection` GUI-bound message.
    CopyProtection(ProtectionState),

    /// The `registration` GUI-bound message.
    Registration(ProtectionState),

    /// The `option` GUI-bound message.
    Option(UciOptionConfig),

    /// The `info` GUI-bound message.
    Info(Vec<UciInfoAttribute>),

    /// Indicating unknown message.
    Unknown(String, Option<PestError<Rule>>),
}

impl UciMessage {
    /// Constructs a `register later` [UciMessage::Register](enum.UciMessage.html#variant.Register)  message.
    pub fn register_later() -> UciMessage {
        UciMessage::Register {
            later: true,
            name: None,
            code: None,
        }
    }

    /// Constructs a `register <code> <name>` [UciMessage::Register](enum.UciMessage.html#variant.Register) message.
    pub fn register_code(name: &str, code: &str) -> UciMessage {
        UciMessage::Register {
            later: false,
            name: Some(name.to_string()),
            code: Some(code.to_string()),
        }
    }

    /// Constructs an empty [UciMessage::Register](enum.UciMessage.html#variant.Go) message.
    pub fn go() -> UciMessage {
        UciMessage::Go {
            search_control: None,
            time_control: None,
        }
    }

    /// Construct a `go ponder` [UciMessage::Register](enum.UciMessage.html#variant.Go) message.
    pub fn go_ponder() -> UciMessage {
        UciMessage::Go {
            search_control: None,
            time_control: Some(UciTimeControl::Ponder),
        }
    }

    /// Constructs a `go infinite` [UciMessage::Register](enum.UciMessage.html#variant.Go) message.
    pub fn go_infinite() -> UciMessage {
        UciMessage::Go {
            search_control: None,
            time_control: Some(UciTimeControl::Infinite),
        }
    }

    /// Constructs a `go movetime <milliseconds>` [UciMessage::Register](enum.UciMessage.html#variant.Go) message, with
    /// `milliseconds` as the argument.
    pub fn go_movetime(milliseconds: Duration) -> UciMessage {
        UciMessage::Go {
            search_control: None,
            time_control: Some(UciTimeControl::MoveTime(milliseconds)),
        }
    }

    /// Constructs an `id <name>` GUI-bound message.
    pub fn id_name(name: &str) -> UciMessage {
        UciMessage::Id {
            name: Some(name.to_string()),
            author: None,
        }
    }

    /// Constructs an `id <name>` GUI-bound message.
    pub fn id_author(author: &str) -> UciMessage {
        UciMessage::Id {
            name: None,
            author: Some(author.to_string()),
        }
    }

    /// Constructs a `bestmove` GUI-bound message without the ponder move.
    #[cfg(not(feature = "chess"))]
    pub fn best_move(best_move: UciMove) -> UciMessage {
        UciMessage::BestMove {
            best_move,
            ponder: None,
        }
    }

    /// Constructs a `bestmove` GUI-bound message _with_ the ponder move.
    #[cfg(not(feature = "chess"))]
    pub fn best_move_with_ponder(best_move: UciMove, ponder: UciMove) -> UciMessage {
        UciMessage::BestMove {
            best_move,
            ponder: Some(ponder),
        }
    }

    /// Constructs a `bestmove` GUI-bound message without the ponder move.
    #[cfg(feature = "chess")]
    pub fn best_move(best_move: ChessMove) -> UciMessage {
        UciMessage::BestMove {
            best_move,
            ponder: None,
        }
    }

    /// Constructs a `bestmove` GUI-bound message _with_ the ponder move.
    #[cfg(feature = "chess")]
    pub fn best_move_with_ponder(best_move: ChessMove, ponder: ChessMove) -> UciMessage {
        UciMessage::BestMove {
            best_move,
            ponder: Some(ponder),
        }
    }

    /// Constructs an `info string ...` message.
    pub fn info_string(s: String) -> UciMessage {
        UciMessage::Info(vec![UciInfoAttribute::String(s)])
    }

    /// Returns whether the command was meant for the engine or for the GUI.
    pub fn direction(&self) -> CommunicationDirection {
        match self {
            UciMessage::Uci
            | UciMessage::Debug(..)
            | UciMessage::IsReady
            | UciMessage::Register { .. }
            | UciMessage::Position { .. }
            | UciMessage::SetOption { .. }
            | UciMessage::UciNewGame
            | UciMessage::Stop
            | UciMessage::PonderHit
            | UciMessage::Quit
            | UciMessage::Go { .. } => CommunicationDirection::GuiToEngine,
            _ => CommunicationDirection::EngineToGui,
        }
    }

    /// If this `UciMessage` is a `UciMessage::SetOption` and the value of that option is a `bool`, this method returns
    /// the `bool` value, otherwise it returns `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            UciMessage::SetOption { value, .. } => {
                if let Some(val) = value {
                    let pr = str::parse(val.as_str());
                    if let Ok(v) = pr {
                        return Some(v);
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// If this `UciMessage` is a `UciMessage::SetOption` and the value of that option is an integer, this method
    /// returns the `i32` value of the integer, otherwise it returns `None`.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            UciMessage::SetOption { value, .. } => {
                if let Some(val) = value {
                    let pr = str::parse(val.as_str());
                    if let Ok(v) = pr {
                        return Some(v);
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// Return `true` if this `UciMessage` is of variant `UnknownMessage`.
    pub fn is_unknown(&self) -> bool {
        matches!(self, UciMessage::Unknown(..))
    }
}

impl Display for UciMessage {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.uci_serialize())
    }
}

impl UciSerializable for UciMessage {
    /// Serializes the command into a String.
    ///
    /// # Examples
    /// ```
    /// use vampirc_uci::{UciMessage, UciSerializable};
    ///
    /// println!("{}", UciMessage::Uci.uci_serialize()); // Should print `uci`.
    /// ```
    fn uci_serialize(&self) -> String {
        match self {
            UciMessage::Debug(on) => {
                if *on {
                    String::from("debug on")
                } else {
                    String::from("debug off")
                }
            }
            UciMessage::Register { later, name, code } => {
                if *later {
                    return String::from("register later");
                }

                let mut s: String = String::from("register ");
                if let Some(n) = name {
                    s += format!("name {}", *n).as_str();
                    if code.is_some() {
                        s += " ";
                    }
                }
                if let Some(c) = code {
                    s += format!("code {}", *c).as_str();
                }

                s
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                let mut s = String::from("position ");
                if *startpos {
                    s += String::from("startpos").as_str();
                } else if let Some(uci_fen) = fen {
                    s += format!("fen {}", uci_fen.as_str()).as_str();
                }

                if !moves.is_empty() {
                    s += String::from(" moves").as_str();

                    for m in moves {
                        s += format!(" {}", *m).as_str();
                    }
                }

                s
            }
            UciMessage::SetOption { name, value } => {
                let mut s: String = format!("setoption name {}", name);

                if let Some(val) = value {
                    if val.is_empty() {
                        s += " value <empty>";
                    } else {
                        s += format!(" value {}", *val).as_str();
                    }
                } else {
                    s += " value <empty>";
                }

                s
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => {
                let mut s = String::from("go ");

                if let Some(tc) = time_control {
                    match tc {
                        UciTimeControl::Infinite => {
                            s += "infinite ";
                        }
                        UciTimeControl::Ponder => {
                            s += "ponder ";
                        }
                        UciTimeControl::MoveTime(duration) => {
                            s += format!("movetime {} ", duration.num_milliseconds()).as_str();
                        }
                        UciTimeControl::TimeLeft {
                            white_time,
                            black_time,
                            white_increment,
                            black_increment,
                            moves_to_go,
                        } => {
                            if let Some(wt) = white_time {
                                s += format!("wtime {} ", wt.num_milliseconds()).as_str();
                            }

                            if let Some(bt) = black_time {
                                s += format!("btime {} ", bt.num_milliseconds()).as_str();
                            }

                            if let Some(wi) = white_increment {
                                s += format!("winc {} ", wi.num_milliseconds()).as_str();
                            }

                            if let Some(bi) = black_increment {
                                s += format!("binc {} ", bi.num_milliseconds()).as_str();
                            }

                            if let Some(mtg) = moves_to_go {
                                s += format!("movestogo {} ", *mtg).as_str();
                            }
                        }
                    }
                }

                if let Some(sc) = search_control {
                    if let Some(depth) = sc.depth {
                        s += format!("depth {} ", depth).as_str();
                    }

                    if let Some(nodes) = sc.nodes {
                        s += format!("nodes {} ", nodes).as_str();
                    }

                    if let Some(mate) = sc.mate {
                        s += format!("mate {} ", mate).as_str();
                    }

                    if !sc.search_moves.is_empty() {
                        s += " searchmoves ";
                        for m in &sc.search_moves {
                            s += format!("{} ", m).as_str();
                        }
                    }
                }

                s
            }
            UciMessage::Uci => "uci".to_string(),
            UciMessage::IsReady => "isready".to_string(),
            UciMessage::UciNewGame => "ucinewgame".to_string(),
            UciMessage::Stop => "stop".to_string(),
            UciMessage::PonderHit => "ponderhit".to_string(),
            UciMessage::Quit => "quit".to_string(),

            // GUI-bound from this point on
            UciMessage::Id { name, author } => {
                let mut s = String::from("id ");
                if let Some(n) = name {
                    s += "name ";
                    s += n;
                } else if let Some(a) = author {
                    s += "author ";
                    s += a;
                }

                s
            }
            UciMessage::UciOk => String::from("uciok"),
            UciMessage::ReadyOk => String::from("readyok"),
            UciMessage::BestMove { best_move, ponder } => {
                let mut s = format!("bestmove {}", *best_move);

                if let Some(p) = ponder {
                    s += format!(" ponder {}", *p).as_str();
                }

                s
            }
            UciMessage::CopyProtection(cp_state) | UciMessage::Registration(cp_state) => {
                let mut s = match self {
                    UciMessage::CopyProtection(..) => String::from("copyprotection "),
                    UciMessage::Registration(..) => String::from("registration "),
                    _ => unreachable!(),
                };

                match cp_state {
                    ProtectionState::Checking => s += "checking",
                    ProtectionState::Ok => s += "ok",
                    ProtectionState::Error => s += "error",
                }

                s
            }
            UciMessage::Option(config) => config.uci_serialize(),
            UciMessage::Info(info_line) => {
                let mut s = String::from("info");

                for a in info_line {
                    s += &format!(" {}", a.uci_serialize());
                }

                s
            }
            UciMessage::Unknown(msg, ..) => {
                format!("UNKNOWN MESSAGE: {}", msg)
            }
        }
    }
}

/// This enum represents the possible variants of the `go` UCI message that deal with the chess game's time controls
/// and the engine's thinking time.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciTimeControl {
    /// The `go ponder` message.
    Ponder,

    /// The `go infinite` message.
    Infinite,

    /// The information about the game's time controls.
    TimeLeft {
        /// White's time on the clock, in milliseconds.
        white_time: Option<Duration>,

        /// Black's time on the clock, in milliseconds.
        black_time: Option<Duration>,

        /// White's increment per move, in milliseconds.
        white_increment: Option<Duration>,

        /// Black's increment per move, in milliseconds.
        black_increment: Option<Duration>,

        /// The number of moves to go to the next time control.
        moves_to_go: Option<u8>,
    },

    /// Specifies how much time the engine should think about the move, in milliseconds.
    MoveTime(Duration),
}

impl UciTimeControl {
    /// Returns a `UciTimeControl::TimeLeft` with all members set to `None`.
    pub fn time_left() -> UciTimeControl {
        UciTimeControl::TimeLeft {
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
        }
    }
}

/// A struct that controls the engine's (non-time-related) search settings.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct UciSearchControl {
    /// Limits the search to these moves.
    #[cfg(not(feature = "chess"))]
    pub search_moves: Vec<UciMove>,

    /// Limits the search to these moves.
    #[cfg(feature = "chess")]
    pub search_moves: Vec<ChessMove>,

    /// Search for mate in this many moves.
    pub mate: Option<u32>,

    /// Search to this ply depth.
    pub depth: Option<u32>,

    /// Search no more than this many nodes (positions).
    pub nodes: Option<u64>,
}

impl UciSearchControl {
    /// Creates an `UciSearchControl` with `depth` set to the parameter and everything else set to empty or `None`.
    pub fn depth(depth: u32) -> UciSearchControl {
        UciSearchControl {
            search_moves: vec![],
            mate: None,
            depth: Some(depth),
            nodes: None,
        }
    }

    /// Creates an `UciSearchControl` with `mate` set to the parameter and everything else set to empty or `None`.
    pub fn mate(mate: u32) -> UciSearchControl {
        UciSearchControl {
            search_moves: vec![],
            mate: Some(mate),
            depth: None,
            nodes: None,
        }
    }

    /// Creates an `UciSearchControl` with `nodes` set to the parameter and everything else set to empty or `None`.
    pub fn nodes(nodes: u64) -> UciSearchControl {
        UciSearchControl {
            search_moves: vec![],
            mate: None,
            depth: None,
            nodes: Some(nodes),
        }
    }

    /// Returns `true` if all of the struct's settings are either `None` or empty.
    pub fn is_empty(&self) -> bool {
        self.search_moves.is_empty()
            && self.mate.is_none()
            && self.depth.is_none()
            && self.nodes.is_none()
    }
}

impl Default for UciSearchControl {
    /// Creates an empty `UciSearchControl`.
    fn default() -> Self {
        UciSearchControl {
            search_moves: vec![],
            mate: None,
            depth: None,
            nodes: None,
        }
    }
}

/// Represents the copy protection or registration state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ProtectionState {
    /// Signifies the engine is checking the copy protection or registration.
    Checking,

    /// Signifies the copy protection or registration has been validated.
    Ok,

    /// Signifies error in copy protection or registratin validation.
    Error,
}

/// Represents a UCI option definition.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "specta", derive(Type))]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "type", content = "value", rename_all = "camelCase")
)]
pub enum UciOptionConfig {
    /// The option of type `check` (a boolean).
    Check {
        /// The name of the option.
        name: String,

        /// The default value of this `bool` property.
        default: Option<bool>,
    },

    /// The option of type `spin` (a signed integer).
    Spin {
        /// The name of the option.
        name: String,

        /// The default value of this integer property.
        default: Option<i64>,

        /// The minimal value of this integer property.
        min: Option<i64>,

        /// The maximal value of this integer property.
        max: Option<i64>,
    },

    /// The option of type `combo` (a list of strings).
    Combo {
        /// The name of the option.
        name: String,

        /// The default value for this list of strings.
        default: Option<String>,

        /// The list of acceptable strings.
        var: Vec<String>,
    },

    /// The option of type `button` (an action).
    Button {
        /// The name of the option.
        name: String,
    },

    /// The option of type `string` (a string, unsurprisingly).
    String {
        /// The name of the option.
        name: String,

        /// The default value of this string option.
        default: Option<String>,
    },
}

impl UciOptionConfig {
    /// Returns the name of the option.
    pub fn get_name(&self) -> &str {
        match self {
            UciOptionConfig::Check { name, .. }
            | UciOptionConfig::Spin { name, .. }
            | UciOptionConfig::Combo { name, .. }
            | UciOptionConfig::Button { name }
            | UciOptionConfig::String { name, .. } => name.as_str(),
        }
    }

    /// Returns the type string of the option (ie. `"check"`, `"spin"` ...)
    pub fn get_type_str(&self) -> &'static str {
        match self {
            UciOptionConfig::Check { .. } => "check",
            UciOptionConfig::Spin { .. } => "spin",
            UciOptionConfig::Combo { .. } => "combo",
            UciOptionConfig::Button { .. } => "button",
            UciOptionConfig::String { .. } => "string",
        }
    }
}

impl UciSerializable for UciOptionConfig {
    /// Serializes this option config into a full UCI message string.
    ///
    /// # Examples
    ///
    /// ```
    /// use vampirc_uci::{UciMessage, UciOptionConfig, UciSerializable};
    ///
    /// let m = UciMessage::Option(UciOptionConfig::Check {
    ///     name: String::from("Nullmove"),
    ///     default: Some(true)
    /// });
    ///
    /// assert_eq!(m.uci_serialize(), "option name Nullmove type check default true");
    /// ```
    fn uci_serialize(&self) -> String {
        let mut s = format!(
            "option name {} type {}",
            self.get_name(),
            self.get_type_str()
        );
        match self {
            UciOptionConfig::Check { default, .. } => {
                if let Some(def) = default {
                    s += format!(" default {}", *def).as_str();
                }
            }
            UciOptionConfig::Spin {
                default, min, max, ..
            } => {
                if let Some(def) = default {
                    s += format!(" default {}", *def).as_str();
                }

                if let Some(m) = min {
                    s += format!(" min {}", *m).as_str();
                }

                if let Some(m) = max {
                    s += format!(" max {}", *m).as_str();
                }
            }
            UciOptionConfig::Combo { default, var, .. } => {
                if let Some(def) = default {
                    s += format!(" default {}", *def).as_str();
                }

                for v in var {
                    s += format!(" var {}", *v).as_str();
                }
            }
            UciOptionConfig::String { default, .. } => {
                if let Some(def) = default {
                    s += format!(" default {}", *def).as_str();
                }
            }
            UciOptionConfig::Button { .. } => {
                // Do nothing, we're already good
            }
        }

        s
    }
}

impl Display for UciOptionConfig {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.uci_serialize())
    }
}

/// The representation of various info messages. For an info attribute that is not listed in the protocol specification,
/// the `UciInfoAttribute::Any(name, value)` variant can be used.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciInfoAttribute {
    /// The `info depth` message.
    Depth(u32),

    /// The `info seldepth` message.
    SelDepth(u32),

    /// The `info time` message.
    Time(Duration),

    /// The `info nodes` message.
    Nodes(u64),

    /// The `info pv` message (best line move sequence).
    #[cfg(not(feature = "chess"))]
    Pv(Vec<UciMove>),

    /// The `info pv` message (best line move sequence).
    #[cfg(feature = "chess")]
    Pv(Vec<ChessMove>),

    /// The `info pv ... multipv` message (the pv line number in a multi pv sequence).
    MultiPv(u16),

    /// The `info score ...` message.
    Score {
        /// The score in centipawns.
        cp: Option<i32>,

        /// Mate coming up in this many moves. Negative value means the engine is getting mated.
        mate: Option<i32>,

        /// The probability of each result (win, draw, loss).
        wdl: Option<(i32, i32, i32)>,

        /// The value sent is the lower bound.
        lower_bound: Option<bool>,

        /// The value sent is the upper bound.
        upper_bound: Option<bool>,
    },

    /// The `info currmove` message (current move).
    #[cfg(not(feature = "chess"))]
    CurrMove(UciMove),

    /// The `info currmove` message (current move).
    #[cfg(feature = "chess")]
    CurrMove(ChessMove),

    /// The `info currmovenum` message (current move number).
    CurrMoveNum(u16),

    /// The `info hashfull` message (the occupancy of hashing tables in permills).
    HashFull(u16),

    /// The `info nps` message (nodes per second).
    Nps(u64),

    /// The `info tbhits` message (end-game table-base hits).
    TbHits(u64),

    /// The `info sbhits` message (I guess some Shredder-specific end-game table-base stuff. I dunno, probably best to
    /// ignore).
    SbHits(u64),

    /// The `info cpuload` message (CPU load in permills).
    CpuLoad(u16),

    /// The `info string` message (a string the GUI should display).
    String(String),

    /// The `info refutation` message (the first move is the move being refuted).
    #[cfg(not(feature = "chess"))]
    Refutation(Vec<UciMove>),

    /// The `info refutation` message (the first move is the move being refuted).
    #[cfg(feature = "chess")]
    Refutation(Vec<ChessMove>),

    /// The `info currline` message (current line being calculated on a CPU).
    CurrLine {
        /// The CPU number calculating this line.
        cpu_nr: Option<u16>,

        /// The line being calculated.
        #[cfg(not(feature = "chess"))]
        line: Vec<UciMove>,

        /// The line being calculated.
        #[cfg(feature = "chess")]
        line: Vec<ChessMove>,
    },

    /// Any other info line in the format `(name, value)`.
    Any(String, String),
}

impl UciInfoAttribute {
    /// Creates a `UciInfoAttribute::Score` with the `cp` attribute set to the value of the parameter and all other
    /// fields set to `None`.
    pub fn from_centipawns(cp: i32) -> UciInfoAttribute {
        UciInfoAttribute::Score {
            cp: Some(cp),
            mate: None,
            wdl: None,
            lower_bound: None,
            upper_bound: None,
        }
    }

    /// Creates a `UciInfoAttribute::Score` with the `mate` attribute set to the value of the parameter and all other
    /// fields set to `None`. A negative value indicates it is the engine that is getting mated.
    pub fn from_mate(mate: i32) -> UciInfoAttribute {
        UciInfoAttribute::Score {
            cp: None,
            mate: Some(mate),
            wdl: None,
            lower_bound: None,
            upper_bound: None,
        }
    }

    /// Returns the name of the info attribute.
    pub fn get_name(&self) -> &str {
        match self {
            UciInfoAttribute::Depth(..) => "depth",
            UciInfoAttribute::SelDepth(..) => "seldepth",
            UciInfoAttribute::Time(..) => "time",
            UciInfoAttribute::Nodes(..) => "nodes",
            UciInfoAttribute::Pv(..) => "pv",
            UciInfoAttribute::MultiPv(..) => "multipv",
            UciInfoAttribute::Score { .. } => "score",
            UciInfoAttribute::CurrMove(..) => "currmove",
            UciInfoAttribute::CurrMoveNum(..) => "currmovenum",
            UciInfoAttribute::HashFull(..) => "hashfull",
            UciInfoAttribute::Nps(..) => "nps",
            UciInfoAttribute::TbHits(..) => "tbhits",
            UciInfoAttribute::SbHits(..) => "sbhits",
            UciInfoAttribute::CpuLoad(..) => "cpuload",
            UciInfoAttribute::String(..) => "string",
            UciInfoAttribute::Refutation(..) => "refutation",
            UciInfoAttribute::CurrLine { .. } => "currline",
            UciInfoAttribute::Any(name, ..) => name.as_str(),
        }
    }
}

impl UciSerializable for UciInfoAttribute {
    /// Returns the attribute serialized as a String.
    fn uci_serialize(&self) -> String {
        let mut s = self.get_name().to_string();
        match self {
            UciInfoAttribute::Depth(depth) => s += format!(" {}", *depth).as_str(),
            UciInfoAttribute::SelDepth(depth) => s += format!(" {}", *depth).as_str(),
            UciInfoAttribute::Time(time) => s += format!(" {}", time.num_milliseconds()).as_str(),
            UciInfoAttribute::Nodes(nodes) => s += format!(" {}", *nodes).as_str(),
            UciInfoAttribute::Pv(moves) | UciInfoAttribute::Refutation(moves) => {
                if !moves.is_empty() {
                    for m in moves {
                        s += format!(" {}", m).as_str();
                    }
                }
            }
            UciInfoAttribute::MultiPv(num) => s += format!(" {}", *num).as_str(),
            UciInfoAttribute::Score {
                cp,
                mate,
                wdl,
                lower_bound,
                upper_bound,
            } => {
                if let Some(c) = cp {
                    s += format!(" cp {}", *c).as_str();
                }

                if let Some(m) = mate {
                    s += format!(" mate {}", *m).as_str();
                }

                if let Some((w, d, l)) = wdl {
                    s += format!(" wdl {} {} {}", *w, *d, *l).as_str();
                }

                if lower_bound.is_some() {
                    s += " lowerbound";
                } else if upper_bound.is_some() {
                    s += " upperbound";
                }
            }
            UciInfoAttribute::CurrMove(uci_move) => s += &format!(" {}", *uci_move),
            UciInfoAttribute::CurrMoveNum(num) => s += &format!(" {}", *num),
            UciInfoAttribute::HashFull(permill) => s += &format!(" {}", *permill),
            UciInfoAttribute::Nps(nps) => s += &format!(" {}", *nps),
            UciInfoAttribute::TbHits(hits) | UciInfoAttribute::SbHits(hits) => {
                s += &format!(" {}", *hits)
            }
            UciInfoAttribute::CpuLoad(load) => s += &format!(" {}", *load),
            UciInfoAttribute::String(string) => s += &format!(" {}", string),
            UciInfoAttribute::CurrLine { cpu_nr, line } => {
                if let Some(c) = cpu_nr {
                    s += &format!(" cpunr {}", *c);
                }

                if !line.is_empty() {
                    for m in line {
                        s += &format!(" {}", m);
                    }
                }
            }
            UciInfoAttribute::Any(_, value) => {
                s += &format!(" {}", value);
            }
        }

        s
    }
}

impl Display for UciInfoAttribute {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.uci_serialize())
    }
}

/// An enum representing the chess piece types.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[cfg(not(feature = "chess"))]
pub enum UciPiece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[cfg(not(feature = "chess"))]
impl UciPiece {
    /// Returns a character representing a piece in UCI move notation. Used for specifying promotion in moves.
    ///
    /// `n` – knight
    /// `b` - bishop
    /// `r` - rook
    /// `q` - queen
    /// `k` - king
    /// `None` - pawn
    pub fn as_char(self) -> Option<char> {
        match self {
            UciPiece::Pawn => None,
            UciPiece::Knight => Some('n'),
            UciPiece::Bishop => Some('b'),
            UciPiece::Rook => Some('r'),
            UciPiece::Queen => Some('q'),
            UciPiece::King => Some('k'),
        }
    }
}

#[cfg(not(feature = "chess"))]
impl FromStr for UciPiece {
    type Err = FmtError;

    /// Creates a `UciPiece` from a `&str`, according to these rules:
    ///
    /// `"n"` - Knight
    /// `"p"` - Pawn
    /// `"b"` - Bishop
    /// `"r"` - Rook
    /// `"k"` - King
    /// `"q"` - Queen
    ///
    /// Works with uppercase letters as well.
    fn from_str(s: &str) -> Result<UciPiece, FmtError> {
        match s.to_ascii_lowercase().as_str() {
            "n" => Ok(UciPiece::Knight),
            "p" => Ok(UciPiece::Pawn),
            "b" => Ok(UciPiece::Bishop),
            "r" => Ok(UciPiece::Rook),
            "k" => Ok(UciPiece::King),
            "q" => Ok(UciPiece::Queen),
            _ => Err(FmtError),
        }
    }
}

/// A representation of a chessboard square.
#[cfg(not(feature = "chess"))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UciSquare {
    /// The file. A character in the range of `a..h`.
    pub file: char,

    /// The rank. A number in the range of `1..8`.
    pub rank: u8,
}

#[cfg(not(feature = "chess"))]
impl UciSquare {
    /// Create a `UciSquare` from file character and a rank number.
    pub fn from(file: char, rank: u8) -> UciSquare {
        UciSquare { file, rank }
    }
}

#[cfg(not(feature = "chess"))]
impl Display for UciSquare {
    /// Formats the square in the regular notation (as in, `e4`).
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}{}", self.file, self.rank)
    }
}

#[cfg(not(feature = "chess"))]
impl Default for UciSquare {
    /// Default square is an invalid square with a file of `\0` and the rank of `0`.
    fn default() -> Self {
        UciSquare {
            file: '\0',
            rank: 0,
        }
    }
}

/// Representation of a chess move.
#[cfg(not(feature = "chess"))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UciMove {
    /// The source square.
    pub from: UciSquare,

    /// The destination square.
    pub to: UciSquare,

    /// The piece to be promoted to, if any.
    pub promotion: Option<UciPiece>,
}

#[cfg(not(feature = "chess"))]
impl UciMove {
    /// Create a regular, non-promotion move from the `from` square to the `to` square.
    pub fn from_to(from: UciSquare, to: UciSquare) -> UciMove {
        UciMove {
            from,
            to,
            promotion: None,
        }
    }
}

#[cfg(not(feature = "chess"))]
impl Display for UciMove {
    /// Formats the move in the UCI move notation.
    ///
    /// `e2e4` – A move from the square `e2` to the square `e4`.
    /// `a2a1q` – A move from the square `a2` to the square `a1` with the pawn promoting to a Queen..
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut r = write!(f, "{}{}", self.from, self.to);

        if let Some(p) = self.promotion {
            if let Some(c) = p.as_char() {
                r = write!(f, "{}", c);
            }
        }

        r
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
/// A representation of the notation in the [FEN notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
pub struct UciFen(pub String);

impl UciFen {
    /// Returns the FEN string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for UciFen {
    /// Constructs an UciFen object from a `&str` containing a [FEN](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation)
    /// position. Does not validate the FEN.
    fn from(s: &str) -> Self {
        UciFen(s.to_string())
    }
}

impl Display for UciFen {
    /// Outputs the FEN string.
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

/// A vector containing several `UciMessage`s.
pub type MessageList = Vec<UciMessage>;

/// A wrapper that keeps the serialized form in a byte vector. Mostly useful to provide an `AsRef<[u8]>` implementation for
/// quick conversion to an array of bytes. Use the `::from(m: UciMessage)` to construct it. It will add the newline
/// character `\n` to the serialized message.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct ByteVecUciMessage {
    pub message: UciMessage,
    pub bytes: Vec<u8>,
}

impl Display for ByteVecUciMessage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}", self.message)
    }
}

impl From<UciMessage> for ByteVecUciMessage {
    fn from(m: UciMessage) -> Self {
        let b = Vec::from((m.uci_serialize() + "\n").as_bytes());
        ByteVecUciMessage {
            message: m,
            bytes: b,
        }
    }
}

impl From<ByteVecUciMessage> for UciMessage {
    fn from(val: ByteVecUciMessage) -> Self {
        val.message
    }
}

impl AsRef<UciMessage> for ByteVecUciMessage {
    fn as_ref(&self) -> &UciMessage {
        &self.message
    }
}

impl AsRef<[u8]> for ByteVecUciMessage {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "chess")]
    use chess::Square;

    use super::*;

    #[test]
    fn test_direction_engine_bound() {
        assert_eq!(
            UciMessage::PonderHit.direction(),
            CommunicationDirection::GuiToEngine
        );
    }

    #[test]
    fn test_direction_gui_bound() {
        assert_eq!(
            UciMessage::UciOk.direction(),
            CommunicationDirection::EngineToGui
        );
    }

    #[test]
    fn test_serialize_id_name() {
        assert_eq!(
            UciMessage::id_name("Vampirc 0.5.0")
                .uci_serialize()
                .as_str(),
            "id name Vampirc 0.5.0"
        );
    }

    #[test]
    fn test_serialize_id_author() {
        assert_eq!(
            UciMessage::id_author("Matija Kejžar")
                .uci_serialize()
                .as_str(),
            "id author Matija Kejžar"
        );
    }

    #[test]
    fn test_serialize_uciok() {
        assert_eq!(UciMessage::UciOk.uci_serialize().as_str(), "uciok");
    }

    #[test]
    fn test_serialize_readyok() {
        assert_eq!(UciMessage::ReadyOk.uci_serialize().as_str(), "readyok");
    }

    #[cfg(not(feature = "chess"))]
    #[test]
    fn test_serialize_bestmove() {
        assert_eq!(
            UciMessage::best_move(UciMove::from_to(
                UciSquare::from('a', 1),
                UciSquare::from('a', 7)
            ))
            .uci_serialize()
            .as_str(),
            "bestmove a1a7"
        );
    }

    #[cfg(feature = "chess")]
    #[test]
    fn test_serialize_bestmove() {
        assert_eq!(
            UciMessage::best_move(ChessMove::new(Square::A1, Square::A7, None))
                .uci_serialize()
                .as_str(),
            "bestmove a1a7"
        );
    }

    #[cfg(not(feature = "chess"))]
    #[test]
    fn test_serialize_bestmove_with_options() {
        assert_eq!(
            UciMessage::best_move_with_ponder(
                UciMove::from_to(UciSquare::from('b', 4), UciSquare::from('a', 5)),
                UciMove::from_to(UciSquare::from('b', 4), UciSquare::from('d', 6))
            )
            .uci_serialize()
            .as_str(),
            "bestmove b4a5 ponder b4d6"
        );
    }

    #[cfg(feature = "chess")]
    #[test]
    fn test_serialize_bestmove_with_options() {
        assert_eq!(
            UciMessage::best_move_with_ponder(
                ChessMove::new(Square::B4, Square::A5, None),
                ChessMove::new(Square::B4, Square::D6, None),
            )
            .uci_serialize()
            .as_str(),
            "bestmove b4a5 ponder b4d6"
        );
    }

    #[test]
    fn test_serialize_copyprotection() {
        assert_eq!(
            UciMessage::CopyProtection(ProtectionState::Checking)
                .uci_serialize()
                .as_str(),
            "copyprotection checking"
        );
    }

    #[test]
    fn test_serialize_registration() {
        assert_eq!(
            UciMessage::Registration(ProtectionState::Ok)
                .uci_serialize()
                .as_str(),
            "registration ok"
        );
    }

    #[test]
    fn test_serialize_check_option() {
        let m = UciMessage::Option(UciOptionConfig::Check {
            name: "Nullmove".to_string(),
            default: Some(false),
        });

        assert_eq!(
            m.uci_serialize(),
            "option name Nullmove type check default false"
        );
    }

    #[test]
    fn test_serialize_spin_option() {
        let m = UciMessage::Option(UciOptionConfig::Spin {
            name: "Selectivity".to_string(),
            default: Some(2),
            min: Some(0),
            max: Some(4),
        });

        assert_eq!(
            m.uci_serialize(),
            "option name Selectivity type spin default 2 min 0 max 4"
        );
    }

    #[test]
    fn test_serialize_combo_option() {
        let m = UciMessage::Option(UciOptionConfig::Combo {
            name: "Style".to_string(),
            default: Some(String::from("Normal")),
            var: vec![
                String::from("Solid"),
                String::from("Normal"),
                String::from("Risky"),
            ],
        });

        assert_eq!(
            m.uci_serialize(),
            "option name Style type combo default Normal var Solid var Normal var Risky"
        );
    }

    #[test]
    fn test_serialize_string_option() {
        let m = UciMessage::Option(UciOptionConfig::String {
            name: "Nalimov Path".to_string(),
            default: Some(String::from("c:\\")),
        });

        assert_eq!(
            m.uci_serialize(),
            "option name Nalimov Path type string default c:\\"
        );
    }

    #[test]
    fn test_serialize_button_option() {
        let m = UciMessage::Option(UciOptionConfig::Button {
            name: "Clear Hash".to_string(),
        });

        assert_eq!(m.uci_serialize(), "option name Clear Hash type button");
    }

    #[test]
    fn test_serialize_info_depth() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Depth(24)];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info depth 24");
    }

    #[test]
    fn test_serialize_info_seldepth() {
        let attributes: Vec<UciInfoAttribute> =
            vec![UciInfoAttribute::Depth(22), UciInfoAttribute::SelDepth(17)];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info depth 22 seldepth 17");
    }

    // info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv e2e4 e7e5 g1f3
    #[test]
    fn test_serialize_info_pv() {
        let attributes: Vec<UciInfoAttribute> = vec![
            UciInfoAttribute::Depth(2),
            UciInfoAttribute::from_centipawns(214),
            UciInfoAttribute::Time(Duration::milliseconds(1242)),
            UciInfoAttribute::Nodes(2124),
            UciInfoAttribute::Nps(34928),
            #[cfg(not(feature = "chess"))]
            UciInfoAttribute::Pv(vec![
                UciMove::from_to(UciSquare::from('e', 2), UciSquare::from('e', 4)),
                UciMove::from_to(UciSquare::from('e', 7), UciSquare::from('e', 5)),
                UciMove::from_to(UciSquare::from('g', 1), UciSquare::from('f', 3)),
            ]),
            #[cfg(feature = "chess")]
            UciInfoAttribute::Pv(vec![
                ChessMove::new(Square::E2, Square::E4, None),
                ChessMove::new(Square::E7, Square::E5, None),
                ChessMove::new(Square::G1, Square::F3, None),
            ]),
        ];

        let m = UciMessage::Info(attributes);

        assert_eq!(
            m.uci_serialize(),
            "info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv e2e4 e7e5 g1f3"
        );
    }

    // info depth 5 seldepth 5 multipv 1 score cp -5 nodes 1540 nps 54 tbhits 0 time 28098 pv a8b6 e3b6 b1b6 a5a7 e2e3
    #[test]
    fn test_serialize_info_multipv() {
        let attributes: Vec<UciInfoAttribute> = vec![
            UciInfoAttribute::Depth(5),
            UciInfoAttribute::SelDepth(5),
            UciInfoAttribute::MultiPv(1),
            UciInfoAttribute::from_centipawns(-5),
            UciInfoAttribute::Nodes(1540),
            UciInfoAttribute::Nps(54),
            UciInfoAttribute::TbHits(0),
            UciInfoAttribute::Time(Duration::milliseconds(28098)),
            #[cfg(not(feature = "chess"))]
            UciInfoAttribute::Pv(vec![
                UciMove::from_to(UciSquare::from('a', 8), UciSquare::from('b', 6)),
                UciMove::from_to(UciSquare::from('e', 3), UciSquare::from('b', 6)),
                UciMove::from_to(UciSquare::from('b', 1), UciSquare::from('b', 6)),
                UciMove::from_to(UciSquare::from('a', 5), UciSquare::from('a', 7)),
                UciMove::from_to(UciSquare::from('e', 2), UciSquare::from('e', 3)),
            ]),
            #[cfg(feature = "chess")]
            UciInfoAttribute::Pv(vec![
                ChessMove::new(Square::A8, Square::B6, None),
                ChessMove::new(Square::E3, Square::B6, None),
                ChessMove::new(Square::B1, Square::B6, None),
                ChessMove::new(Square::A5, Square::A7, None),
                ChessMove::new(Square::E2, Square::E3, None),
            ]),
        ];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info depth 5 seldepth 5 multipv 1 score cp -5 nodes 1540 nps 54 tbhits 0 time 28098 pv a8b6 e3b6 b1b6 a5a7 e2e3");
    }

    #[test]
    fn test_serialize_info_score() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Score {
            cp: Some(817),
            mate: None,
            wdl: None,
            upper_bound: Some(true),
            lower_bound: None,
        }];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info score cp 817 upperbound");
    }

    #[test]
    fn test_serialize_info_score_mate_in_three() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Score {
            cp: None,
            mate: Some(-3),
            wdl: None,
            upper_bound: None,
            lower_bound: None,
        }];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info score mate -3");
    }

    #[test]
    fn test_serialize_info_currmove() {
        #[cfg(not(feature = "chess"))]
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::CurrMove(UciMove::from_to(
            UciSquare::from('a', 5),
            UciSquare::from('c', 3),
        ))];

        #[cfg(feature = "chess")]
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::CurrMove(ChessMove::new(
            Square::A5,
            Square::C3,
            None,
        ))];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info currmove a5c3");
    }

    #[test]
    fn test_serialize_info_currmovenum() {
        #[cfg(not(feature = "chess"))]
        let attributes: Vec<UciInfoAttribute> = vec![
            UciInfoAttribute::CurrMove(UciMove::from_to(
                UciSquare::from('a', 2),
                UciSquare::from('f', 2),
            )),
            UciInfoAttribute::CurrMoveNum(2),
        ];

        #[cfg(feature = "chess")]
        let attributes: Vec<UciInfoAttribute> = vec![
            UciInfoAttribute::CurrMove(ChessMove::new(Square::A2, Square::F2, None)),
            UciInfoAttribute::CurrMoveNum(2),
        ];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info currmove a2f2 currmovenum 2");
    }

    #[test]
    fn test_serialize_info_hashfull() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::HashFull(455)];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info hashfull 455");
    }

    #[test]
    fn test_serialize_info_nps() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Nps(5098)];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info nps 5098");
    }

    #[test]
    fn test_serialize_info_tbhits_nbhits() {
        let attributes: Vec<UciInfoAttribute> =
            vec![UciInfoAttribute::TbHits(987), UciInfoAttribute::SbHits(409)];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info tbhits 987 sbhits 409");
    }

    #[test]
    fn test_serialize_info_cpuload() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::CpuLoad(823)];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info cpuload 823");
    }

    #[test]
    fn test_serialize_info_string() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::String(String::from(
            "Invalid move: d6e1 - violates chess rules",
        ))];

        let m = UciMessage::Info(attributes);

        assert_eq!(
            m.uci_serialize(),
            "info string Invalid move: d6e1 - violates chess rules"
        );
    }

    #[test]
    fn test_serialize_info_refutation() {
        #[cfg(not(feature = "chess"))]
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Refutation(vec![
            UciMove::from_to(UciSquare::from('d', 1), UciSquare::from('h', 5)),
            UciMove::from_to(UciSquare::from('g', 6), UciSquare::from('h', 5)),
        ])];

        #[cfg(feature = "chess")]
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Refutation(vec![
            ChessMove::new(Square::D1, Square::H5, None),
            ChessMove::new(Square::G6, Square::H5, None),
        ])];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info refutation d1h5 g6h5");
    }

    #[test]
    fn test_serialize_info_currline() {
        #[cfg(not(feature = "chess"))]
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::CurrLine {
            cpu_nr: Some(1),
            line: vec![
                UciMove::from_to(UciSquare::from('d', 1), UciSquare::from('h', 5)),
                UciMove::from_to(UciSquare::from('g', 6), UciSquare::from('h', 5)),
            ],
        }];

        #[cfg(feature = "chess")]
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::CurrLine {
            cpu_nr: Some(1),
            line: vec![
                ChessMove::new(Square::D1, Square::H5, None),
                ChessMove::new(Square::G6, Square::H5, None),
            ],
        }];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info currline cpunr 1 d1h5 g6h5");
    }

    #[test]
    fn test_serialize_info_any() {
        let attributes: Vec<UciInfoAttribute> = vec![UciInfoAttribute::Any(
            String::from("other"),
            String::from("Some other message."),
        )];

        let m = UciMessage::Info(attributes);

        assert_eq!(m.uci_serialize(), "info other Some other message.");
    }

    #[test]
    fn test_serialize_none_setoption() {
        assert_eq!(
            UciMessage::SetOption {
                name: "Some option".to_string(),
                value: None,
            }
            .uci_serialize(),
            "setoption name Some option value <empty>"
        )
    }

    #[test]
    fn test_serialize_empty_setoption() {
        assert_eq!(
            UciMessage::SetOption {
                name: "ABC".to_string(),
                value: Some(String::from("")),
            }
            .uci_serialize(),
            "setoption name ABC value <empty>"
        )
    }

    #[test]
    fn test_is_unknown_false() {
        assert!(!UciMessage::Uci.is_unknown());
    }

    #[test]
    fn test_is_unknown_true() {
        let um = UciMessage::Unknown("Unrecognized Command".to_owned(), None);
        assert!(um.is_unknown());
    }

    #[test]
    fn test_byte_vec_message_creation() {
        let uok = ByteVecUciMessage::from(UciMessage::UciOk);
        assert_eq!(uok.message, UciMessage::UciOk);
        assert_eq!(
            uok.bytes,
            (UciMessage::UciOk.uci_serialize() + "\n").as_bytes()
        );

        let asm: UciMessage = uok.into();
        assert_eq!(asm, UciMessage::UciOk);
    }

    #[test]
    fn test_byte_vec_message_as_ref_uci_message() {
        let uci = ByteVecUciMessage::from(UciMessage::Uci);
        let um: &UciMessage = uci.as_ref();
        assert_eq!(*um, UciMessage::Uci);
    }

    #[test]
    fn test_byte_vec_message_as_ref_u8() {
        let uci = ByteVecUciMessage::from(UciMessage::UciNewGame);
        let um: &[u8] = uci.as_ref();
        let uc = Vec::from(um);
        assert_eq!(
            uc,
            Vec::from((UciMessage::UciNewGame.uci_serialize() + "\n").as_bytes())
        );
    }

    #[test]
    fn test_empty_go_message() {
        let empty_go = UciMessage::go();
        assert_eq!(
            empty_go,
            UciMessage::Go {
                time_control: None,
                search_control: None
            }
        );
    }

    #[test]
    fn test_negative_duration() {
        let time_control = UciTimeControl::TimeLeft {
            white_time: Some(Duration::milliseconds(-4061)),
            black_time: Some(Duration::milliseconds(56826)),
            white_increment: None,
            black_increment: None,
            moves_to_go: Some(90),
        };

        let message = UciMessage::Go {
            time_control: Some(time_control),
            search_control: None,
        };

        match message {
            UciMessage::Go {
                time_control,
                search_control: _,
            } => {
                let tc = time_control.unwrap();
                match tc {
                    UciTimeControl::TimeLeft {
                        white_time,
                        black_time,
                        white_increment: _,
                        black_increment: _,
                        moves_to_go: _,
                    } => {
                        let wt = white_time.unwrap();
                        assert_eq!(wt, Duration::milliseconds(-4061));
                        assert_eq!(wt.num_milliseconds(), -4061);
                        assert_eq!(wt.num_seconds(), -4);
                        assert_eq!(black_time.unwrap(), Duration::milliseconds(56826));
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
