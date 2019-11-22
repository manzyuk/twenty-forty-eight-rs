use cursive::event::Key;
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

fn show_game_state(state: &GameState) -> String {
    [
        show_board(&state.board),
        format!("Score: {:4}", state.score),
    ]
    .concat()
}

fn show_board(board: &Board) -> String {
    let mut hline = String::new();
    hline.push('+');
    for _ in 0..board.size {
        hline.push_str("------");
        hline.push('+');
    }
    let mut s = String::new();
    s.push_str(&hline);
    s.push('\n');
    for row in board.rows.iter() {
        s.push_str(&show_row(row));
        s.push('\n');
        s.push_str(&hline);
        s.push('\n');
    }
    s
}

fn show_row(row: &Vec<Tile>) -> String {
    let n = row.len();
    let mut top = String::new();
    top.push('|');
    for _ in 0..n {
        top.push_str("      ");
        top.push('|');
    }
    let bot = top.clone();
    let mut mid = String::new();
    mid.push('|');
    for tile in row.iter() {
        mid.push_str(&show_tile(tile));
        mid.push('|');
    }
    [top, "\n".to_owned(), mid, "\n".to_owned(), bot].concat()
}

fn show_tile(tile: &Tile) -> String {
    match *tile {
        Tile::Blank => "      ".to_owned(),
        Tile::Number(k) => format!(" {:4} ", k),
    }
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

#[test]
fn test_transpose() {
    assert_eq!(
        transpose(vec![
            vec![Tile::Number(1), Tile::Number(2)],
            vec![Tile::Number(3), Tile::Number(4)],
        ]),
        vec![
            vec![Tile::Number(1), Tile::Number(3)],
            vec![Tile::Number(2), Tile::Number(4)],
        ],
    );
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
