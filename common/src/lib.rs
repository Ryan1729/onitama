extern crate rand;

use std::fmt;

use rand::StdRng;

pub struct Platform {
    pub print_xy: fn(i32, i32, &str),
    pub print_xy_offset: fn(i32, i32, i32, i32, &str),
    pub clear: fn(Option<Rect>),
    pub size: fn() -> Size,
    pub pick: fn(Point, i32) -> char,
    pub mouse_position: fn() -> Point,
    pub clicks: fn() -> i32,
    pub key_pressed: fn(KeyCode) -> bool,
    pub set_colors: fn(Color, Color),
    pub get_colors: fn() -> (Color, Color),
    pub set_foreground: fn(Color),
    pub get_foreground: fn() -> (Color),
    pub set_background: fn(Color),
    pub get_background: fn() -> (Color),
    pub set_layer: fn(i32),
    pub get_layer: fn() -> i32,
}

pub struct State {
    pub rng: StdRng,
    pub board: Board,
    pub player_cards: (Card, Card),
    pub center_card: Card,
    pub cpu_cards: (Card, Card),
    pub turn: Turn,
    pub ui_context: UIContext,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Turn {
    Waiting,
    SelectedCard(PairIndex),
    SelectedPiece(PairIndex, usize),
    CpuTurn,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PairIndex {
    First,
    Second,
}

pub trait AllValues {
    fn all_values() -> Vec<Self>
    where
        Self: std::marker::Sized;
}

#[derive(Debug, Copy, Clone)]
pub enum Card {
    Tiger,
    Crab,
    Monkey,
    Crane,
    Dragon,
    Elephant,
    Mantis,
    Boar,
    Frog,
    Goose,
    Horse,
    Eel,
    Rabbit,
    Rooster,
    Ox,
    Cobra,
}
use Card::*;

impl Card {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Tiger => "\u{E0C0}",
            Crab => "\u{E0C1}",
            Monkey => "\u{E0C2}",
            Crane => "\u{E0C3}",
            Dragon => "\u{E0C4}",
            Elephant => "\u{E0C5}",
            Mantis => "\u{E0C6}",
            Boar => "\u{E0C7}",
            Frog => "\u{E0C8}",
            Goose => "\u{E0C9}",
            Horse => "\u{E0CA}",
            Eel => "\u{E0CB}",
            Rabbit => "\u{E0CC}",
            Rooster => "\u{E0CD}",
            Ox => "\u{E0CE}",
            Cobra => "\u{E0CF}",
        }
    }
}

impl AllValues for Card {
    fn all_values() -> Vec<Card> {
        vec![
            Tiger,
            Crab,
            Monkey,
            Crane,
            Dragon,
            Elephant,
            Mantis,
            Boar,
            Frog,
            Goose,
            Horse,
            Eel,
            Rabbit,
            Rooster,
            Ox,
            Cobra,
        ]
    }
}

pub type Board = [Option<Piece>; 25];

pub fn valid_move_locations(
    board: &Board,
    card: &Card,
    piece_index: usize,
    piece_colour: PieceColour,
) -> Vec<(usize, usize)> {
    let mut result = Vec::new();

    if is_index_on_board(piece_index) {
        if let Some((x, y)) = get_board_xy(piece_index) {
            for &(x_1, y_1) in get_offsets(card).iter() {
                let result_x = (x as isize).wrapping_add(x_1) as usize;
                let result_y = (y as isize).wrapping_add(y_1) as usize;
                if let Some(target_index) = get_board_index(result_x, result_y) {
                    if board[target_index].map(Piece::piece_colour) != Some(piece_colour) {
                        result.push((result_x, result_y));
                    }
                }
            }
        }
    }

    result
}

fn get_offsets(card: &Card) -> Vec<(isize, isize)> {
    match *card {
        Tiger => vec![(0, -2), (0, 1)],
        Crab => vec![(-2, 0), (2, 0), (0, -1)],
        Monkey => vec![(-1, -1), (-1, 1), (1, -1), (1, 1)],
        Crane => vec![(-1, 1), (1, 1), (0, -1)],
        Dragon => vec![(-1, 1), (1, 1), (-2, -1), (2, -1)],
        Elephant => vec![(-1, 0), (1, 0), (-1, -1), (1, -1)],
        Mantis => vec![(-1, -1), (-1, 1), (0, 1)],
        Boar => vec![(-1, 0), (1, 0), (0, -1)],
        Frog => vec![(-2, 0), (-1, -1), (1, 1)],
        Goose => vec![(-1, 0), (-1, -1), (1, 0), (1, 1)],
        Horse => vec![(-1, 0), (0, -1), (0, 1)],
        Eel => vec![(1, 0), (-1, -1), (-1, 1)],
        Rabbit => vec![(2, 0), (1, -1), (-1, 1)],
        Rooster => vec![(1, 0), (1, -1), (-1, 0), (-1, 1)],
        Ox => vec![(1, 0), (0, -1), (0, 1)],
        Cobra => vec![(-1, 0), (1, -1), (1, 1)],
    }
}

pub fn get_board_index(x: usize, y: usize) -> Option<usize> {
    let result = y * 5 + x;

    if is_index_on_board(result) {
        Some(result)
    } else {
        None
    }
}

pub fn get_board_xy(index: usize) -> Option<(usize, usize)> {
    if is_index_on_board(index) {
        Some((index % 5, index / 5))
    } else {
        None
    }
}

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod board_indices {
    use get_board_index;
    use get_board_xy;

    quickcheck! {
      fn prop(i: usize) -> bool {
          i == get_board_xy(i).and_then(|(x,y)|
            get_board_index(x,y)
      ).unwrap_or(i)
      }
  }
}

fn is_index_on_board(piece_index: usize) -> bool {
    piece_index < 25
}

pub const TOP_PAGODA_INDEX: usize = 2;
pub const BOTTOM_PAGODA_INDEX: usize = 22;

#[derive(Copy, Clone, Debug)]
pub enum Piece {
    RedStudent,
    BlueStudent,
    RedMaster,
    BlueMaster,
}
use Piece::*;

impl Piece {
    pub fn is_player(&self) -> bool {
        match *self {
            BlueStudent | BlueMaster => true,
            _ => false,
        }
    }
    pub fn piece_colour(self) -> PieceColour {
        match self {
            BlueStudent | BlueMaster => Blue,
            RedStudent | RedMaster => Red,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceColour {
    Red,
    Blue,
}
use PieceColour::*;

pub type UiId = i32;

pub struct UIContext {
    pub hot: UiId,
    pub active: UiId,
    pub next_hot: UiId,
}

impl UIContext {
    pub fn new() -> Self {
        UIContext {
            hot: 0,
            active: 0,
            next_hot: 0,
        }
    }

    pub fn set_not_active(&mut self) {
        self.active = 0;
    }
    pub fn set_active(&mut self, id: UiId) {
        self.active = id;
    }
    pub fn set_next_hot(&mut self, id: UiId) {
        self.next_hot = id;
    }
    pub fn set_not_hot(&mut self) {
        self.hot = 0;
    }
    pub fn frame_init(&mut self) {
        if self.active == 0 {
            self.hot = self.next_hot;
        }
        self.next_hot = 0;
    }
}








//NOTE(Ryan1729): if I import BearLibTerminal.rs into `state_manipulation` or a crate
//`state_manipulation` depends on, like this one for example, then the
//ffi to the C version of BearLibTerminal causes an error. I just want
//the geometry datatypes and the Event and Keycode definitions so I have
//copied them from BearLibTerminal.rs below

//BearLibTerminal.rs is released under the MIT license by nabijaczleweli.
//see https://github.com/nabijaczleweli/BearLibTerminal.rs/blob/master/LICENSE
//for full details.

impl Point {
    /// Creates a new point on the specified non-negative coordinates
    pub fn new_safe(mut x: i32, mut y: i32) -> Point {
        x = if x >= 0 { x } else { 0 };
        y = if y >= 0 { y } else { 0 };

        Point { x: x, y: y }
    }

    pub fn add(&self, x: i32, y: i32) -> Point {
        Point::new_safe(self.x + x, self.y + y)
    }
}

/// Represents a single on-screen point/coordinate pair.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Creates a new point on the specified non-negative coordinates
    pub fn new(x: i32, y: i32) -> Point {
        assert!(x >= 0);
        assert!(y >= 0);

        Point { x: x, y: y }
    }
}


/// A 2D size representation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    /// Creates a new non-negative size.
    pub fn new(width: i32, height: i32) -> Size {
        assert!(width >= 0);
        assert!(height >= 0);

        Size {
            width: width,
            height: height,
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}x{}", self.width, self.height)
    }
}

/// A rectangle, described by its four corners and a size.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Rect {
    /// The top-left corner.
    pub top_left: Point,
    /// The top-right corner.
    pub top_right: Point,
    /// The bottom-right corner.
    pub bottom_right: Point,
    /// The bottom-left corner.
    pub bottom_left: Point,
    /// The `Rect`angle's size.
    pub size: Size,
}

impl Rect {
    /// Construct a `Rect` from its top-left corner and its size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use common::{Rect, Point, Size};
    /// let rect = Rect::from_size(Point::new(10, 20), Size::new(30, 40));
    /// assert_eq!(rect.top_left, Point::new(10, 20));
    /// assert_eq!(rect.top_right, Point::new(40, 20));
    /// assert_eq!(rect.bottom_left, Point::new(10, 60));
    /// assert_eq!(rect.bottom_right, Point::new(40, 60));
    /// assert_eq!(rect.size, Size::new(30, 40));
    /// ```
    pub fn from_size(origin: Point, size: Size) -> Rect {
        let top_right = Point::new(origin.x + size.width, origin.y);
        let bottom_left = Point::new(origin.x, origin.y + size.height);
        let bottom_right = Point::new(top_right.x, bottom_left.y);

        Rect {
            top_left: origin,
            top_right: top_right,
            bottom_left: bottom_left,
            bottom_right: bottom_right,
            size: size,
        }
    }

    /// Construct a `Rect` from its top-left and bottom-right corners.
    ///
    /// # Examples
    ///
    /// ```
    /// # use common::{Rect, Point, Size};
    /// let rect = Rect::from_points(Point::new(10, 20), Point::new(30, 40));
    /// assert_eq!(rect.top_left, Point::new(10, 20));
    /// assert_eq!(rect.top_right, Point::new(30, 20));
    /// assert_eq!(rect.bottom_left, Point::new(10, 40));
    /// assert_eq!(rect.bottom_right, Point::new(30, 40));
    /// assert_eq!(rect.size, Size::new(20, 20));
    /// ```
    pub fn from_points(top_left: Point, bottom_right: Point) -> Rect {
        assert!(bottom_right.x >= top_left.x);
        assert!(bottom_right.y >= top_left.y);

        let size = Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y);
        Rect::from_size(top_left, size)
    }

    /// Construct a `Rect` from its top-left corner and its size, values unwrapped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use common::{Rect, Point, Size};
    /// assert_eq!(Rect::from_values(10, 20, 30, 40),
    ///     Rect::from_size(Point::new(10, 20), Size::new(30, 40)));
    /// ```
    pub fn from_values(x: i32, y: i32, width: i32, height: i32) -> Rect {
        let origin = Point::new(x, y);
        let size = Size::new(width, height);
        Rect::from_size(origin, size)
    }


    /// Construct a `Rect` from its top-left and bottom-right corners, values unwrapped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use common::{Rect, Point, Size};
    /// assert_eq!(Rect::from_point_values(10, 20, 30, 40),
    ///     Rect::from_points(Point::new(10, 20), Point::new(30, 40)));
    /// ```
    pub fn from_point_values(
        top_left_x: i32,
        top_left_y: i32,
        bottom_right_x: i32,
        bottom_right_y: i32,
    ) -> Rect {
        let top_left = Point::new(top_left_x, top_left_y);
        let bottom_right = Point::new(bottom_right_x, bottom_right_y);
        Rect::from_points(top_left, bottom_right)
    }
}

//input module

/// All pressable keys.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// Top-row `1/!` key.
    Row1,
    /// Top-row `2/@` key.
    Row2,
    /// Top-row `3/#` key.
    Row3,
    /// Top-row `4/$` key.
    Row4,
    /// Top-row `5/%` key.
    Row5,
    /// Top-row `6/^` key.
    Row6,
    /// Top-row `7/&` key.
    Row7,
    /// Top-row `8/*` key.
    Row8,
    /// Top-row `9/(` key.
    Row9,
    /// Top-row `0/)` key.
    Row0,
    /// Top-row &#96;/~ key.
    Grave,
    /// Top-row `-/_` key.
    Minus,
    /// Top-row `=/+` key.
    Equals,
    /// Second-row `[/{` key.
    LeftBracket,
    /// Second-row `]/}` key.
    RightBracket,
    /// Second-row `\/|` key.
    Backslash,
    /// Third-row `;/:` key.
    Semicolon,
    /// Third-row `'/"` key.
    Apostrophe,
    /// Fourth-row `,/<` key.
    Comma,
    /// Fourth-row `./>` key.
    Period,
    /// Fourth-row `//?` key.
    Slash,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
    Pause,
    Insert,
    Home,
    PageUp,
    Delete,
    End,
    PageDown,
    /// Right arrow key.
    Right,
    /// Left arrow key.
    Left,
    /// Down arrow key.
    Down,
    /// Up arrow key.
    Up,
    /// Numpad `/` key.
    NumDivide,
    /// Numpad `*` key.
    NumMultiply,
    /// Numpad `-` key.
    NumMinus,
    /// Numpad `+` key.
    NumPlus,
    /// Numpad &#9166; key.
    NumEnter,
    /// Numpad `Del/.` key (output locale-dependent).
    NumPeriod,
    /// Numpad `1/End` key.
    Num1,
    /// Numpad 2/&#8595; key.
    Num2,
    /// Numpad `3/PageDown` key.
    Num3,
    /// Numpad 4/&#8592; key.
    Num4,
    /// Numpad `5` key.
    Num5,
    /// Numpad 6/&#8594; key.
    Num6,
    /// Numpad `7/Home` key.
    Num7,
    /// Numpad 8/&#8593; key.
    Num8,
    /// Numpad `9/PageUp` key.
    Num9,
    /// Numpad `0/Insert` key.
    Num0,
    /// Left mouse button.
    MouseLeft,
    /// Right mouse button.
    MouseRight,
    /// Middle mouse button a.k.a. pressed scroll wheel.
    MouseMiddle,
    MouseFourth,
    MouseFifth,
}

/// A single input event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    /// Terminal window closed.
    Close,
    /// Terminal window resized. Needs to have `window.resizeable = true` to occur.
    ///
    /// Note, that the terminal window is cleared when resized.
    Resize {
        /// Width the terminal was resized to.
        width: i32,
        /// Heigth the terminal was resized to.
        height: i32,
    },
    /// Mouse moved.
    ///
    /// If [`precise-mouse`](config/struct.Input.html#structfield.precise_mouse) is off,
    /// generated each time mouse moves from cell to cell, otherwise,
    /// when it moves from pixel to pixel.
    MouseMove {
        /// `0`-based cell index from the left to which the mouse cursor moved.
        x: i32,
        /// `0`-based cell index from the top to which the mouse cursor moved.
        y: i32,
    },
    /// Mouse wheel moved.
    MouseScroll {
        /// Amount of steps the wheel rotated.
        ///
        /// Positive when scrolled "down"/"backwards".
        ///
        /// Negative when scrolled "up"/"forwards"/"away".
        delta: i32,
    },
    /// A keyboard or mouse button pressed (might repeat, if set in OS).
    KeyPressed {
        /// The key pressed.
        key: KeyCode,
        /// Whether the Control key is pressed.
        ctrl: bool,
        /// Whether the Shift key is pressed.
        shift: bool,
    },
    /// A keyboard or mouse button released.
    KeyReleased {
        /// The key released.
        key: KeyCode,
        /// Whether the Control key is pressed.
        ctrl: bool,
        /// Whether the Shift key is pressed.
        shift: bool,
    },
    /// The Shift key pressed (might repeat, if set in OS).
    ShiftPressed,
    /// The Shift key released.
    ShiftReleased,
    /// The Shift key pressed (might repeat, if set in OS).
    ControlPressed,
    /// The Control key released.
    ControlReleased,
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
