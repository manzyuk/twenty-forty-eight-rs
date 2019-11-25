use cursive::event::Key;
use cursive::theme::Color;
use cursive::theme::ColorStyle;
use cursive::theme::ColorType;
use cursive::theme::Style;
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, TextView};
use cursive::Cursive;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    play(&mut Cursive::default());
}

fn play(siv: &mut Cursive) {
    let state = initial_game_state();

    siv.add_layer(TextView::new(show_game_state(&state)));

    let state_ref = Rc::new(RefCell::new(state));
    siv.add_global_callback(Key::Esc, |s| s.quit());
    siv.add_global_callback(Key::Up, make_callback(&state_ref, slide_up));
    siv.add_global_callback(Key::Down, make_callback(&state_ref, slide_down));
    siv.add_global_callback(Key::Left, make_callback(&state_ref, slide_left));
    siv.add_global_callback(Key::Right, make_callback(&state_ref, slide_right));

    siv.run();
}

fn make_callback<F>(state_ref: &Rc<RefCell<GameState>>, f: F) -> impl FnMut(&mut Cursive) + 'static
where
    F: Fn(GameState) -> GameState + 'static,
{
    let state_ref = Rc::clone(&state_ref);
    move |s| {
        let mut state = state_ref.borrow_mut();
        *state = add_random_tile(f((*state).clone()));
        s.pop_layer();
        s.add_layer(TextView::new(show_game_state(&*state)));
        if is_complete(&*state) {
            end_game_or_replay(s, "You win!");
        } else if is_stuck(&*state) {
            end_game_or_replay(s, "Game over!");
        }
    }
}

fn end_game_or_replay(siv: &mut Cursive, status: &str) {
    siv.add_layer(
        Dialog::text("Try again?")
            .title(status)
            .button("Yes", |s| {
                s.pop_layer();
                s.pop_layer();
                play(s);
            })
            .button("No", |s| s.quit()),
    );
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Tile {
    Blank,
    Number(u32),
}

#[derive(Debug, PartialEq)]
struct Board {
    rows: Vec<Vec<Tile>>,
    size: usize,
}

#[derive(Debug, PartialEq)]
struct GameState {
    board: Board,
    score: u32,
}

impl Clone for Board {
    fn clone(&self) -> Board {
        Board {
            rows: self.rows.clone(),
            size: self.size,
        }
    }
}

impl Clone for GameState {
    fn clone(&self) -> GameState {
        GameState {
            board: self.board.clone(),
            score: self.score,
        }
    }
}

const BORDER_COLOR: Color = Color::RgbLowRes(2, 2, 2);

fn color_string(s: String, fc: Color, bc: Color) -> StyledString {
    StyledString::styled(
        s,
        Style::from(ColorStyle::new(ColorType::from(fc), ColorType::from(bc))),
    )
}

fn show_game_state(state: &GameState) -> StyledString {
    let mut s = StyledString::new();
    s.append(show_board(&state.board));
    s.append_plain(format!("Score: {:4}", state.score));
    s
}

fn show_board(board: &Board) -> StyledString {
    let line = join(
        color_string(" ".to_owned(), BORDER_COLOR, BORDER_COLOR),
        vec![color_string("──────".to_owned(), BORDER_COLOR, BORDER_COLOR); board.size],
    );

    let mut s = StyledString::new();
    let mut first = true;
    s.append(line.clone());
    for row in board.rows.iter() {
        if !first {
            s.append(line.clone());
        }
        s.append_plain("\n");
        s.append(show_row(row));
        s.append_plain("\n");
        first = false;
    }
    s.append(line.clone());
    s.append_plain("\n");
    s
}

fn show_row(row: &Vec<Tile>) -> StyledString {
    let colors: Vec<Color> = row.iter().map(|tile| tile_bc(tile)).collect();
    let top = join(
        color_string(" ".to_owned(), BORDER_COLOR, BORDER_COLOR),
        colors
            .into_iter()
            .map(|color| color_string("      ".to_owned(), color, color))
            .collect(),
    );
    let mid = join(
        color_string(" ".to_owned(), BORDER_COLOR, BORDER_COLOR),
        row.into_iter().map(|tile| show_tile(tile)).collect(),
    );
    let bot = top.clone();

    let mut s = StyledString::new();
    s.append(top);
    s.append_plain("\n");
    s.append(mid);
    s.append_plain("\n");
    s.append(bot);
    s
}

fn show_tile(tile: &Tile) -> StyledString {
    color_string(
        match *tile {
            Tile::Blank => "      ".to_owned(),
            Tile::Number(k) => format!(" {:4} ", k),
        },
        tile_fc(tile),
        tile_bc(tile),
    )
}

fn tile_fc(tile: &Tile) -> Color {
    match *tile {
        Tile::Blank => Color::RgbLowRes(3, 3, 3),
        Tile::Number(2) | Tile::Number(4) | Tile::Number(8) | Tile::Number(16) => {
            Color::RgbLowRes(0, 0, 0)
        }
        Tile::Number(32)
        | Tile::Number(64)
        | Tile::Number(128)
        | Tile::Number(256)
        | Tile::Number(512)
        | Tile::Number(1024)
        | Tile::Number(2048)
        | Tile::Number(_) => Color::RgbLowRes(5, 5, 5),
    }
}

fn tile_bc(tile: &Tile) -> Color {
    match *tile {
        Tile::Blank => Color::RgbLowRes(3, 3, 3),
        Tile::Number(2) => Color::RgbLowRes(5, 5, 5),
        Tile::Number(4) => Color::RgbLowRes(5, 5, 4),
        Tile::Number(8) => Color::RgbLowRes(5, 4, 4),
        Tile::Number(16) => Color::RgbLowRes(5, 4, 3),
        Tile::Number(32) => Color::RgbLowRes(5, 3, 3),
        Tile::Number(64) => Color::RgbLowRes(5, 3, 2),
        Tile::Number(128) => Color::RgbLowRes(5, 2, 2),
        Tile::Number(256) => Color::RgbLowRes(5, 2, 1),
        Tile::Number(512) => Color::RgbLowRes(5, 1, 1),
        Tile::Number(1024) => Color::RgbLowRes(5, 1, 0),
        Tile::Number(_) => Color::RgbLowRes(5, 0, 0),
    }
}

fn join(sep: StyledString, pieces: Vec<StyledString>) -> StyledString {
    let mut line = StyledString::new();
    let mut first = true;
    line.append(sep.clone());
    for piece in pieces.into_iter() {
        if !first {
            line.append(sep.clone());
        }
        line.append(piece);
        first = false;
    }
    line.append(sep.clone());
    line
}

fn empty_board(size: usize) -> Board {
    Board {
        rows: vec![vec![Tile::Blank; size]; size],
        size,
    }
}

fn slide_right(state: GameState) -> GameState {
    let (new_rows, scores): (Vec<Vec<Tile>>, Vec<u32>) = state
        .board
        .rows
        .into_iter()
        .map(|row| slide_row_right(row))
        .unzip();
    let new_score = state.score + scores.iter().sum::<u32>();
    GameState {
        board: Board {
            rows: new_rows,
            size: state.board.size,
        },
        score: new_score,
    }
}

fn slide_row_right(row: Vec<Tile>) -> (Vec<Tile>, u32) {
    let n = row.len();
    let numbers: Vec<u32> = row
        .into_iter()
        .filter_map(|tile| match tile {
            Tile::Number(n) => Some(n),
            Tile::Blank => None,
        })
        .collect();
    let (merged_numbers, score) = merge_numbers_right(numbers);
    let mut merged_tiles: Vec<Tile> = merged_numbers
        .into_iter()
        .map(|n| Tile::Number(n))
        .collect();
    while merged_tiles.len() < n {
        merged_tiles.push(Tile::Blank);
    }
    merged_tiles.reverse();
    (merged_tiles, score)
}

fn merge_numbers_right(mut numbers: Vec<u32>) -> (Vec<u32>, u32) {
    let mut merged_numbers = Vec::new();
    let mut score = 0;
    while let Some(k) = numbers.pop() {
        if let Some(&l) = numbers.last() {
            if k == l {
                numbers.pop();
                merged_numbers.push(k + l);
                score += k + l;
            } else {
                merged_numbers.push(k);
            }
        } else {
            merged_numbers.push(k);
        }
    }
    (merged_numbers, score)
}

fn slide_left(state: GameState) -> GameState {
    reflect_board(slide_right(reflect_board(state)))
}

fn reflect_board(state: GameState) -> GameState {
    GameState {
        board: Board {
            rows: state
                .board
                .rows
                .into_iter()
                .map(|row| {
                    let mut new_row = row.clone();
                    new_row.reverse();
                    new_row
                })
                .collect(),
            size: state.board.size,
        },
        score: state.score,
    }
}

fn slide_up(state: GameState) -> GameState {
    transpose_board(slide_left(transpose_board(state)))
}

fn slide_down(state: GameState) -> GameState {
    transpose_board(slide_right(transpose_board(state)))
}

fn transpose_board(state: GameState) -> GameState {
    GameState {
        board: Board {
            rows: transpose(state.board.rows),
            size: state.board.size,
        },
        score: state.score,
    }
}

fn transpose(rows: Vec<Vec<Tile>>) -> Vec<Vec<Tile>> {
    let mut heads = Vec::new();
    let mut tails = Vec::new();
    for row in rows.into_iter() {
        if let Some((&head, tail)) = row.split_first() {
            heads.push(head);
            tails.push(tail.to_vec());
        } else {
            return vec![];
        }
    }
    let mut result = Vec::new();
    result.push(heads);
    result.extend(transpose(tails));
    result
}

fn blank_tile_positions(board: &Board) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    for (row_index, row) in board.rows.iter().enumerate() {
        for (col_index, tile) in row.iter().enumerate() {
            if let Tile::Blank = tile {
                positions.push((row_index, col_index));
            }
        }
    }
    positions
}

fn choose<T>(xs: &Vec<T>) -> Option<&T> {
    if xs.len() == 0 {
        return None;
    }
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let i = rng.gen_range(0, xs.len());
    xs.get(i)
}

fn add_random_tile(state: GameState) -> GameState {
    let mut state = state.clone();
    if let Some(&(row_index, col_index)) = choose(&blank_tile_positions(&state.board)) {
        if let Some(&tile) = choose(&vec![Tile::Number(2), Tile::Number(4)]) {
            state.board = add_tile(state.board, row_index, col_index, tile);
        }
    }
    // If there are no blank tiles, return a copy of the input state.
    state
}

fn add_tile(board: Board, row_index: usize, col_index: usize, new_tile: Tile) -> Board {
    let mut board = board.clone();
    if let Some(row) = board.rows.get_mut(row_index) {
        if let Some(tile) = row.get_mut(col_index) {
            *tile = new_tile;
        }
    }
    board
}

fn initial_game_state() -> GameState {
    add_random_tile(add_random_tile(GameState {
        board: empty_board(4),
        score: 0,
    }))
}

fn is_complete(state: &GameState) -> bool {
    let rows = &state.board.rows;
    rows.into_iter()
        .any(|row| row.into_iter().any(|&tile| tile == Tile::Number(2048)))
}

fn is_stuck(state: &GameState) -> bool {
    let slides: Vec<fn(GameState) -> GameState> =
        vec![slide_up, slide_down, slide_left, slide_right];
    slides
        .into_iter()
        .map(|slide| slide(state.clone()))
        .all(|new_state| new_state == *state)
}
