extern crate rand;
extern crate common;

use common::*;
use common::Piece::*;
use common::PieceColour::*;
use common::Card::*;
use common::Turn::*;
use common::PairIndex::*;

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

const CONTROL_COLOUR: Color = Color {
    red: 0x99,
    green: 0x99,
    blue: 0x99,
    alpha: 0xFF,
};

macro_rules! with_foreground {
    ($platform:expr, $foreground:expr, $code:block) => {
        {
            let current = ($platform.get_foreground)();
            ($platform.set_foreground)($foreground);

            $code

            ($platform.set_foreground)(current);
        }
    }
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

    let mut deck = Card::all_values();
    rng.shuffle(&mut deck);

    debug_assert!(deck.len() >= 5);

    let player_cards = (deck.pop().unwrap(), deck.pop().unwrap());
    let cpu_cards = (deck.pop().unwrap(), deck.pop().unwrap());

    let center_card = deck.pop().unwrap();

    State {
        rng,
        board,
        player_cards,
        center_card,
        cpu_cards,
        turn: Waiting,
        show_credits: false,
        rotate_opponet_cards: true,
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

    let first_clicked = !state.show_credits &&
        do_card_button(
            platform,
            &mut state.ui_context,
            6,
            32,
            &state.player_cards.0,
            120,
            left_mouse_pressed,
            left_mouse_released,
        );

    let second_clicked = !state.show_credits &&
        do_card_button(
            platform,
            &mut state.ui_context,
            42,
            32,
            &state.player_cards.1,
            121,
            left_mouse_pressed,
            left_mouse_released,
        );

    let possible_board_input = if state.show_credits {
        None
    } else {
        show_pieces(
            platform,
            state,
            400,
            left_mouse_pressed,
            left_mouse_released,
        )
    };

    with_foreground!(platform, CONTROL_COLOUR, {
        let credits_spec = ButtonSpec {
            base: BlankButtonSpec {
                x: 2,
                y: 24,
                w: 16,
                h: 3,
                id: 6,
            },
            text: (if state.show_credits {
                       "Hide credits"
                   } else {
                       "Show credits"
                   }).to_string(),
        };

        if do_button(
            platform,
            &mut state.ui_context,
            &credits_spec,
            left_mouse_pressed,
            left_mouse_released,
        )
        {
            state.show_credits = !state.show_credits;
        }
    });

    if state.show_credits {
        (platform.print_xy)(
            3,
            3,
            "
        ===Image credits===

        \"Ninja Head\" by DarkZaitzev

        \"Ninja Mask\" by Lorc

        \"Pagoda\" by Delapouite

        All of the above icons used under CC BY 3.0.

        the cards use a font called Rokkit,
        which is licensed under the SIL Open Font License

        see README.md for URLs

        ===Everything else===
        Ryan Wiedemann (Ryan1729 on github)",
        );
    } else {
        print_card(
            platform,
            6,
            1,
            &state.cpu_cards.0,
            state.rotate_opponet_cards,
        );
        print_card(
            platform,
            42,
            1,
            &state.cpu_cards.1,
            state.rotate_opponet_cards,
        );

        print_card(platform, 2, 16, &state.center_card, false);

        with_foreground!(platform, CONTROL_COLOUR, {
            let new_game_spec = ButtonSpec {
                base: BlankButtonSpec {
                    x: 2,
                    y: 10,
                    w: 12,
                    h: 3,
                    id: 5,
                },
                text: "New game".to_string(),
            };

            if do_button(
                platform,
                &mut state.ui_context,
                &new_game_spec,
                left_mouse_pressed,
                left_mouse_released,
            )
            {
                *state = make_state(state.rng);
            }

            let rotate_spec = ButtonSpec {
                base: BlankButtonSpec {
                    x: 2,
                    y: 27,
                    w: 30,
                    h: 3,
                    id: 7,
                },
                text: "Rotate opponents's cards".to_string(),
            };

            if do_button(
                platform,
                &mut state.ui_context,
                &rotate_spec,
                left_mouse_pressed,
                left_mouse_released,
            )
            {
                state.rotate_opponet_cards = !state.rotate_opponet_cards;
            }

        });
    }
    let t = state.turn;

    if let Some(board_input) = possible_board_input {
        state.turn = board_input;
    } else {
        match state.turn {
            Waiting => {
                if first_clicked {
                    state.turn = SelectedCard(First);
                } else if second_clicked {
                    state.turn = SelectedCard(Second);
                }
            }
            SelectedCard(pair_index) => {
                if first_clicked {
                    state.turn = SelectedCard(First);
                } else if second_clicked {
                    state.turn = SelectedCard(Second);
                }

                if get_moves(&state.board, &state.player_cards, Blue).len() == 0 {
                    if cfg!(debug_assertions) {
                        println!(
                            "player swapping {:?} for {:?}",
                            state.center_card,
                            match pair_index {
                                First => state.player_cards.0,
                                Second => state.player_cards.1,
                            }
                        );
                    }

                    swap_cards(&mut state.center_card, &mut state.player_cards, pair_index);
                }
            }
            SelectedPiece(pair_index, source_index) => {
                if first_clicked {
                    state.turn = SelectedCard(First);
                } else if second_clicked {
                    state.turn = SelectedCard(Second);
                }

                let mut have_not_moved = true;
                let mut counter = 0;
                for &(x_usize, y_usize) in
                    valid_move_locations(
                        &state.board,
                        match pair_index {
                            First => &state.player_cards.0,
                            Second => &state.player_cards.1,
                        },
                        source_index,
                        Blue,
                    ).iter()
                {
                    let x = x_usize as i32;
                    let y = y_usize as i32;
                    if do_blank_button(
                        platform,
                        &mut state.ui_context,
                        &BlankButtonSpec {
                            x: piece_x(x) - 4,
                            y: piece_y(y) - 2,
                            w: PIECE_BUTTON_WIDTH,
                            h: PIECE_BUTTON_HEIGHT,
                            id: 450 + counter,
                        },
                        left_mouse_pressed,
                        left_mouse_released,
                    ) && have_not_moved
                    {
                        if let Some(target_index) = get_board_index(x_usize, y_usize) {
                            state.board = apply_move(
                                &state.board,
                                Move {
                                    source_index,
                                    target_index,
                                },
                            );

                            if cfg!(debug_assertions) {
                                println!(
                                    "player moving from {} to {} with {:?}",
                                    source_index,
                                    target_index,
                                    match pair_index {
                                        First => state.player_cards.0,
                                        Second => state.player_cards.1,
                                    }
                                );
                            }

                            swap_cards(&mut state.center_card, &mut state.player_cards, pair_index);

                            state.turn = winner(&state.board).unwrap_or(CpuTurn);

                            have_not_moved = false;
                        }

                    }
                    counter += 1;

                    with_layer!(platform, 3, {
                        (platform.print_xy)(piece_x(x), piece_y(y), &BLUE_HIGHLIGHT.to_string());
                    })
                }
            }
            CpuTurn => {
                let moves = get_moves(&state.board, &state.cpu_cards, Red);

                let mut not_moved = true;

                for &(current_move, pair_index) in moves.iter() {
                    let new_board = apply_move(&state.board, current_move);

                    if red_wins(&new_board) {
                        state.board = new_board;

                        swap_cards(&mut state.center_card, &mut state.cpu_cards, pair_index);

                        not_moved = false;
                        break;
                    }
                }

                if not_moved {
                    let mut viable_moves = Vec::new();
                    //these shouldn't contain inviable moves either
                    let mut no_player_capture_moves = Vec::new();
                    let mut capturing_moves = Vec::new();

                    let blue_before = piece_count_of_colour(&state.board, Blue);
                    for &(current_move, pair_index) in moves.iter() {
                        let one_move_board = apply_move(&state.board, current_move);

                        let player_moves = get_moves(&one_move_board, &state.player_cards, Blue);

                        let player_has_no_winning_move =
                            !player_moves.iter().any(|&(player_move, _)| {
                                blue_wins(&apply_move(&one_move_board, player_move))
                            });
                        if player_has_no_winning_move {
                            viable_moves.push((current_move, pair_index));

                            let red_before = piece_count_of_colour(&one_move_board, Red);
                            let player_cannot_capture =
                                !player_moves.iter().any(|&(player_move, _)| {
                                    let after = piece_count_of_colour(
                                        &apply_move(&one_move_board, player_move),
                                        Red,
                                    );

                                    red_before > after
                                });

                            if player_cannot_capture {
                                no_player_capture_moves.push((current_move, pair_index));
                            }

                            let captured = blue_before >
                                piece_count_of_colour(&one_move_board, Blue);

                            if captured {
                                capturing_moves.push((current_move, pair_index));
                            }
                        }
                    }

                    let undefended_captures: Vec<_> = capturing_moves
                        .iter()
                        .filter(|x| no_player_capture_moves.contains(x))
                        .map(|x| *x)
                        .collect();

                    if not_moved && undefended_captures.len() > 0 {
                        make_random_cpu_move_from_vec(state, undefended_captures);
                        not_moved = false;
                    }

                    let agressive = state.rng.gen::<bool>();

                    let (first_priority, second_priority) = if agressive {
                        (capturing_moves, no_player_capture_moves)
                    } else {
                        (no_player_capture_moves, capturing_moves)
                    };

                    if not_moved && first_priority.len() > 0 {
                        make_random_cpu_move_from_vec(state, first_priority);
                        not_moved = false;
                    }

                    if not_moved && second_priority.len() > 0 {
                        make_random_cpu_move_from_vec(state, second_priority);
                        not_moved = false;
                    }

                    if not_moved && viable_moves.len() > 0 {
                        make_random_cpu_move_from_vec(state, viable_moves);
                        not_moved = false;
                    } else if cfg!(debug_assertions) && not_moved {
                        println!("player can win");
                    }
                }

                if not_moved {
                    make_random_cpu_move_from_vec(state, moves);
                }

                state.turn = winner(&state.board).unwrap_or(Waiting);
            }
            Over(winner_colour) => {
                (platform.print_xy)(10, 14, &format!("{} team wins", winner_colour));
            }
        }
    }

    if cfg!(debug_assertions) {
        if t != state.turn {
            println!("{:?}", state.turn);
        }
    }

    false
}

fn piece_count_of_colour(board: &Board, piece_colour: PieceColour) -> usize {
    board
        .iter()
        .filter(|p| {
            p.map(|piece| piece.colour() == piece_colour).unwrap_or(
                false,
            )
        })
        .count()
}

fn make_random_cpu_move_from_vec(state: &mut State, moves: Vec<(Move, PairIndex)>) {
    let len = moves.len();
    if len == 0 {
        let chosen_pair_index = state.rng.gen::<PairIndex>();
        if cfg!(debug_assertions) {
            println!(
                "cpu swapping {:?} for {:?}",
                state.center_card,
                match chosen_pair_index {
                    First => state.cpu_cards.0,
                    Second => state.cpu_cards.1,
                }
            );
        }

        //can't move so just pick a card to switch
        swap_cards(
            &mut state.center_card,
            &mut state.cpu_cards,
            chosen_pair_index,
        );
    } else {
        let (random_move, pair_index) = moves[state.rng.gen_range(0, len)];

        if cfg!(debug_assertions) {
            println!(
                "cpu moving from {} to {} with {:?}",
                random_move.source_index,
                random_move.target_index,
                match pair_index {
                    First => state.cpu_cards.0,
                    Second => state.cpu_cards.1,
                }
            );
        }

        state.board = apply_move(&state.board, random_move);

        swap_cards(&mut state.center_card, &mut state.cpu_cards, pair_index);
    }
}

fn get_moves(board: &Board, cards: &(Card, Card), colour: PieceColour) -> Vec<(Move, PairIndex)> {
    let pieces = get_piece_indices(board, colour);

    let mut result = Vec::new();

    for piece in pieces.iter() {
        result.extend(valid_moves(board, &cards.0, *piece, colour).iter().map(
            |m| {
                (*m, First)
            },
        ));
        result.extend(valid_moves(board, &cards.1, *piece, colour).iter().map(
            |m| {
                (*m, Second)
            },
        ));
    }

    result
}

fn get_piece_indices(board: &Board, colour: PieceColour) -> Vec<usize> {
    board
        .iter()
        .enumerate()
        .filter_map(|(index, piece)| {
            piece.and_then(|p| if p.colour() == colour {
                Some(index)
            } else {
                None
            })

        })
        .collect()
}

fn winner(board: &Board) -> Option<Turn> {
    if blue_wins(board) {
        Some(Over(Blue))
    } else if red_wins(board) {
        Some(Over(Red))
    } else {
        None
    }
}

fn blue_wins(board: &Board) -> bool {
    if let Some(master_index) = get_master_index(board, Blue) {
        master_index == TOP_PAGODA_INDEX || get_master_index(board, Red).is_none()
    } else {
        false
    }
}
fn red_wins(board: &Board) -> bool {
    if let Some(master_index) = get_master_index(board, Red) {
        master_index == BOTTOM_PAGODA_INDEX || get_master_index(board, Blue).is_none()
    } else {
        false
    }
}

fn get_master_index(board: &Board, colour: PieceColour) -> Option<usize> {
    match colour {
        Red => {
            board.iter().position(|p| match *p {
                Some(RedMaster) => true,
                _ => false,
            })
        }
        Blue => {
            board.iter().position(|p| match *p {
                Some(BlueMaster) => true,
                _ => false,
            })
        }
    }
}



//maybe this should also handle card switcing as well?
fn apply_move(board: &Board, current_move: Move) -> Board {
    let mut result = board.clone();

    let piece = result[current_move.source_index];
    result[current_move.target_index] = piece;
    result[current_move.source_index] = None;

    result
}

fn swap_cards(center_card: &mut Card, cards: &mut (Card, Card), pair_index: PairIndex) {
    let swapped = get_swap_cards_result(*center_card, *cards, pair_index);

    *center_card = swapped.center_card;
    *cards = swapped.pair;
}
fn get_swap_cards_result(
    center_card: Card,
    cards: (Card, Card),
    pair_index: PairIndex,
) -> SwapCards {
    match pair_index {
        First => {
            SwapCards {
                center_card: cards.0,
                pair: (center_card, cards.1),
            }
        }
        Second => {
            SwapCards {
                center_card: cards.1,
                pair: (cards.0, center_card),
            }
        }
    }
}

struct SwapCards {
    center_card: Card,
    pair: (Card, Card),
}

fn show_pieces(
    platform: &Platform,
    state: &mut State,
    id_offset: UiId,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
) -> Option<Turn> {
    let mut result = None;

    for y in 0..5 {
        for x in 0..5 {
            (platform.print_xy)(piece_x(x), piece_y(y), &SPACE_EDGE.to_string());

            if let Some(index) = get_board_index(x as usize, y as usize) {

                let i = index as i32;
                if let Some(piece) = state.board[index] {
                    if piece.is_player() {
                        match state.turn {
                            SelectedCard(card) => {
                                if do_piece_button(
                                    platform,
                                    &mut state.ui_context,
                                    x,
                                    y,
                                    piece,
                                    id_offset + i,
                                    left_mouse_pressed,
                                    left_mouse_released,
                                )
                                {
                                    result = Some(SelectedPiece(card, index));
                                }
                            }
                            // // SelectedPiece(_ /*, piece_index*/) => {}
                            _ => {
                                print_piece_xy(platform, x, y, &piece_char(piece).to_string());
                            }
                        }
                    } else {
                        print_piece_xy(platform, x, y, &piece_char(piece).to_string());
                    }
                } else if index == TOP_PAGODA_INDEX {
                    print_piece_xy(platform, x, y, &PAGODA_RED.to_string());
                } else if index == BOTTOM_PAGODA_INDEX {
                    print_piece_xy(platform, x, y, &PAGODA_BLUE.to_string());
                }

            }

        }
    }

    result
}

const PIECE_BUTTON_WIDTH: i32 = 9;
const PIECE_BUTTON_HEIGHT: i32 = 5;

fn do_piece_button(
    platform: &Platform,
    context: &mut UIContext,
    x: i32,
    y: i32,
    piece: Piece,
    id: UiId,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
) -> bool {
    let result = do_blank_button(
        platform,
        context,
        &BlankButtonSpec {
            x: piece_x(x) - 4,
            y: piece_y(y) - 2,
            w: PIECE_BUTTON_WIDTH,
            h: PIECE_BUTTON_HEIGHT,
            id,
        },
        left_mouse_pressed,
        left_mouse_released,
    );

    print_piece_xy(platform, x, y, &piece_char(piece).to_string());

    result
}

fn do_card_button(
    platform: &Platform,
    context: &mut UIContext,
    x: i32,
    y: i32,
    card: &Card,
    id: UiId,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
) -> bool {
    let result = do_blank_button(
        platform,
        context,
        &BlankButtonSpec {
            x,
            y,
            w: CARD_WIDTH,
            h: CARD_HEIGHT,
            id,
        },
        left_mouse_pressed,
        left_mouse_released,
    );

    place_card_tile(platform, x, y, card, false);

    result
}

const CARD_WIDTH: i32 = 32;
const CARD_HEIGHT: i32 = 8;

fn print_card(platform: &Platform, x: i32, y: i32, card: &Card, rotate: bool) {
    draw_rect(platform, x, y, CARD_WIDTH, CARD_HEIGHT);
    place_card_tile(platform, x, y, card, rotate);
}

fn place_card_tile(platform: &Platform, x: i32, y: i32, card: &Card, rotate: bool) {
    with_layer!(platform, 1, {
        (platform.print_xy_offset)(x + 15, y + 3, 0, 7, card.as_str(rotate));
    });
}

fn print_piece_xy(platform: &Platform, x: i32, y: i32, s: &str) {
    with_layer!(platform, 1, {
        (platform.print_xy)(piece_x(x), piece_y(y), s);
    });
}

fn piece_x(x: i32) -> i32 {
    40 + (x * 8)
}
fn piece_y(y: i32) -> i32 {
    12 + (y * 4)
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
const BLUE_HIGHLIGHT: char = '\u{E007}';
const RED_HIGHLIGHT: char = '\u{E008}';

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
    pub base: BlankButtonSpec,
    pub text: String,
}

pub struct BlankButtonSpec {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub id: i32,
}

//calling this once will swallow multiple clicks on the button. We could either
//pass in and return the number of clicks to fix that, or this could simply be
//called multiple times per frame (once for each click).
fn do_blank_button(
    platform: &Platform,
    context: &mut UIContext,
    spec: &BlankButtonSpec,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
) -> bool {
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
        draw_rect_with(
            platform,
            spec.x,
            spec.y,
            spec.w,
            spec.h,
            ["╔", "═", "╕", "║", "│", "╙", "─", "┘"],
        );
    } else if context.hot == id {
        draw_rect_with(
            platform,
            spec.x,
            spec.y,
            spec.w,
            spec.h,
            ["┌", "─", "╖", "│", "║", "╘", "═", "╝"],
        );
    } else {
        draw_rect(platform, spec.x, spec.y, spec.w, spec.h);
    }

    result
}

fn do_button(
    platform: &Platform,
    context: &mut UIContext,
    spec: &ButtonSpec,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
) -> bool {
    let base = &spec.base;

    let result = do_blank_button(
        platform,
        context,
        base,
        left_mouse_pressed,
        left_mouse_released,
    );

    print_centered_line(platform, base.x, base.y, base.w, base.h, &spec.text);

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
    draw_rect_with(
        platform,
        x,
        y,
        w,
        h,
        ["┌", "─", "┐", "│", "│", "└", "─", "┘"],
    );
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
