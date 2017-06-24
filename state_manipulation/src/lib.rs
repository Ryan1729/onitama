extern crate rand;
extern crate common;

use common::*;
use common::Piece::*;

use rand::{StdRng, SeedableRng, Rng};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
#[cfg(debug_assertions)]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //skip the title screen
    println!("debug on");

    let seed: &[_] = &[42];
    let rng: StdRng = SeedableRng::from_seed(seed);

    make_state(rng)
}
#[cfg(not(debug_assertions))]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //show the title screen
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|dur| dur.as_secs())
        .unwrap_or(42);

    println!("{}", timestamp);
    let seed: &[_] = &[timestamp as usize];
    let rng: StdRng = SeedableRng::from_seed(seed);

    make_state(rng)
}

macro_rules! with_layer {
    ($platform:expr, $layer:expr, $code:block) => {
        {
            let current = ($platform.get_layer)();
            ($platform.set_layer)($layer);

            $code

            ($platform.set_layer)(current);
        }
    }
}

fn make_state(mut rng: StdRng) -> State {
    let mut board = [None; 25];
    board[0] = Some(RedStudent);
    board[1] = Some(RedStudent);
    board[2] = Some(RedMaster);
    board[3] = Some(RedStudent);
    board[4] = Some(RedStudent);

    board[20] = Some(BlueStudent);
    board[21] = Some(BlueStudent);
    board[22] = Some(BlueMaster);
    board[23] = Some(BlueStudent);
    board[24] = Some(BlueStudent);

    State {
        rng,
        board,
        ui_context: UIContext::new(),
    }
}

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    let mut left_mouse_pressed = false;
    let mut left_mouse_released = false;

    for event in events {
        cross_mode_event_handling(platform, state, event);

        match *event {
            Event::KeyPressed {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_pressed = true;
            }
            Event::KeyReleased {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_released = true;
            }
            Event::Close |
            Event::KeyPressed {
                key: KeyCode::Escape,
                ctrl: _,
                shift: _,
            } => return true,
            _ => (),
        }
    }

    state.ui_context.frame_init();

    let button_spec = ButtonSpec {
        x: 0,
        y: 0,
        w: 11,
        h: 3,
        text: "Button".to_string(),
        id: 1,
    };

    if do_button(platform,
                 &mut state.ui_context,
                 &button_spec,
                 left_mouse_pressed,
                 left_mouse_released) {
        println!("Button pushed!");
    }
    // println!("{:?}", state.board);
    for y in 0..5 {
        for x in 0..5 {
            print_piece_xy(platform, x, y, &SPACE_EDGE.to_string());

            with_layer!(platform, 1, {
                let i = (y * 5 + x) as usize;
                if let Some(piece) = state.board[i] {
                    print_piece_xy(platform, x, y, &piece_char(piece).to_string());
                } else if i == TOP_PAGODA_INDEX {
                    print_piece_xy(platform, x, y, &PAGODA_RED.to_string());
                } else if i == BOTTOM_PAGODA_INDEX {
                    print_piece_xy(platform, x, y, &PAGODA_BLUE.to_string());
                }
            });

        }
    }

    false
}


fn print_piece_xy(platform: &Platform, x: i32, y: i32, s: &str) {
    (platform.print_xy)(40 + (x * 8), 10 + (y * 4), s);
}

fn piece_char(piece: Piece) -> char {
    match piece {
        RedStudent => STUDENT_RED,
        BlueStudent => STUDENT_BLUE,
        RedMaster => MASTER_RED,
        BlueMaster => MASTER_BLUE,
    }
}

const STUDENT_RED: char = '\u{E000}';
const MASTER_RED: char = '\u{E001}';
const PAGODA_RED: char = '\u{E002}';
const STUDENT_BLUE: char = '\u{E003}';
const MASTER_BLUE: char = '\u{E004}';
const PAGODA_BLUE: char = '\u{E005}';

const SPACE_EDGE: char = '\u{E006}';

fn cross_mode_event_handling(platform: &Platform, state: &mut State, event: &Event) {
    match *event {
        Event::KeyPressed {
            key: KeyCode::R,
            ctrl: true,
            shift: _,
        } => {
            println!("reset");
            *state = new_state((platform.size)());
        }
        _ => (),
    }
}

pub struct ButtonSpec {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub text: String,
    pub id: i32,
}

//calling this once will swallow multiple clicks on the button. We could either
//pass in and return the number of clicks to fix that, or this could simply be
//called multiple times per frame (once for each click).
fn do_button(platform: &Platform,
             context: &mut UIContext,
             spec: &ButtonSpec,
             left_mouse_pressed: bool,
             left_mouse_released: bool)
             -> bool {
    let mut result = false;

    let mouse_pos = (platform.mouse_position)();
    let inside = inside_rect(mouse_pos, spec.x, spec.y, spec.w, spec.h);
    let id = spec.id;

    if context.active == id {
        if left_mouse_released {
            result = context.hot == id && inside;

            context.set_not_active();
        }
    } else if context.hot == id {
        if left_mouse_pressed {
            context.set_active(id);
        }
    }

    if inside {
        context.set_next_hot(id);
    }

    if context.active == id && (platform.key_pressed)(KeyCode::MouseLeft) {
        draw_rect_with(platform,
                       spec.x,
                       spec.y,
                       spec.w,
                       spec.h,
                       ["╔", "═", "╕", "║", "│", "╙", "─", "┘"]);
    } else if context.hot == id {
        draw_rect_with(platform,
                       spec.x,
                       spec.y,
                       spec.w,
                       spec.h,
                       ["┌", "─", "╖", "│", "║", "╘", "═", "╝"]);
    } else {
        draw_rect(platform, spec.x, spec.y, spec.w, spec.h);
    }

    print_centered_line(platform, spec.x, spec.y, spec.w, spec.h, &spec.text);

    return result;
}

pub fn inside_rect(point: Point, x: i32, y: i32, w: i32, h: i32) -> bool {
    x <= point.x && y <= point.y && point.x < x + w && point.y < y + h
}

fn print_centered_line(platform: &Platform, x: i32, y: i32, w: i32, h: i32, text: &str) {
    let x_ = {
        let rect_middle = x + (w / 2);

        rect_middle - (text.chars().count() as f32 / 2.0) as i32
    };

    let y_ = y + (h / 2);

    (platform.print_xy)(x_, y_, &text);
}


fn draw_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(platform,
                   x,
                   y,
                   w,
                   h,
                   ["┌", "─", "┐", "│", "│", "└", "─", "┘"]);
}


fn draw_rect_with(platform: &Platform, x: i32, y: i32, w: i32, h: i32, edges: [&str; 8]) {
    (platform.clear)(Some(Rect::from_values(x, y, w, h)));

    let right = x + w - 1;
    let bottom = y + h - 1;
    // top
    (platform.print_xy)(x, y, edges[0]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, y, edges[1]);
    }
    (platform.print_xy)(right, y, edges[2]);

    // sides
    for i in (y + 1)..bottom {
        (platform.print_xy)(x, i, edges[3]);
        (platform.print_xy)(right, i, edges[4]);
    }

    //bottom
    (platform.print_xy)(x, bottom, edges[5]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, bottom, edges[6]);
    }
    (platform.print_xy)(right, bottom, edges[7]);
}
