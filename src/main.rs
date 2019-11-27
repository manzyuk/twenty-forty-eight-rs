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
    let state = GameState::init();

    siv.add_layer(TextView::new(state.show()));

    let state_ref = Rc::new(RefCell::new(state));
    siv.add_global_callback(Key::Esc, |s| s.quit());
    siv.add_global_callback(Key::Up, make_callback(&state_ref, GameState::slide_up));
    siv.add_global_callback(Key::Down, make_callback(&state_ref, GameState::slide_down));
    siv.add_global_callback(Key::Left, make_callback(&state_ref, GameState::slide_left));
    siv.add_global_callback(
        Key::Right,
        make_callback(&state_ref, GameState::slide_right),
    );

    siv.run();
}

fn make_callback<F>(state_ref: &Rc<RefCell<GameState>>, f: F) -> impl FnMut(&mut Cursive) + 'static
where
    F: Fn(&mut GameState) + 'static,
{
    let state_ref = Rc::clone(&state_ref);
    move |s| {
        let mut state = state_ref.borrow_mut();
        f(&mut state);
        state.add_random_tile();
        s.pop_layer();
        s.add_layer(TextView::new(state.show()));
        if state.is_complete() {
            end_game_or_replay(s, "You win!");
        } else if state.is_stuck() {
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

impl Tile {
    fn foreground_color(&self) -> Color {
        match *self {
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

    fn background_color(&self) -> Color {
        match *self {
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

    fn show(&self) -> StyledString {
        color_string(
            match *self {
                Tile::Blank => "      ".to_owned(),
                Tile::Number(k) => format!(" {:4} ", k),
            },
            self.foreground_color(),
            self.background_color(),
        )
    }
}

#[derive(Debug, PartialEq)]
struct Board {
    rows: Vec<Vec<Tile>>,
    size: usize,
}

impl Board {
    fn empty(size: usize) -> Self {
        Board {
            rows: vec![vec![Tile::Blank; size]; size],
            size,
        }
    }

    fn reflect(&mut self) {
        for row in self.rows.iter_mut() {
            row.reverse();
        }
    }

    fn transpose(&mut self) {
        let old_rows = &self.rows;
        let mut new_rows = vec![vec![]; self.size];
        for row in old_rows.into_iter() {
            for (j, tile) in row.into_iter().enumerate() {
                &new_rows[j].push(*tile);
            }
        }
        self.rows = new_rows;
    }

    fn slide_right(&mut self) -> u32 {
        let old_rows = &self.rows;
        let (new_rows, scores): (Vec<Vec<Tile>>, Vec<u32>) =
            old_rows.iter().map(|row| slide_row_right(row)).unzip();
        self.rows = new_rows;
        scores.iter().sum::<u32>()
    }

    fn slide_left(&mut self) -> u32 {
        self.reflect();
        let score = self.slide_right();
        self.reflect();
        score
    }

    fn slide_up(&mut self) -> u32 {
        self.transpose();
        let score = self.slide_left();
        self.transpose();
        score
    }

    fn slide_down(&mut self) -> u32 {
        self.transpose();
        let score = self.slide_right();
        self.transpose();
        score
    }

    fn add_tile(&mut self, row_index: usize, col_index: usize, new_tile: Tile) {
        if let Some(row) = self.rows.get_mut(row_index) {
            if let Some(tile) = row.get_mut(col_index) {
                *tile = new_tile;
            }
        }
    }

    fn blank_tile_positions(&self) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for (row_index, row) in self.rows.iter().enumerate() {
            for (col_index, tile) in row.iter().enumerate() {
                if let Tile::Blank = tile {
                    positions.push((row_index, col_index));
                }
            }
        }
        positions
    }

    fn add_random_tile(&mut self) {
        if let Some(&(row_index, col_index)) = choose(&self.blank_tile_positions()) {
            if let Some(&tile) = choose(&vec![Tile::Number(2), Tile::Number(4)]) {
                self.add_tile(row_index, col_index, tile);
            }
        }
    }

    fn is_complete(&self) -> bool {
        let rows = &self.rows;
        rows.into_iter()
            .any(|row| row.into_iter().any(|&tile| tile == Tile::Number(2048)))
    }

    fn is_stuck(&self) -> bool {
        for slide in &[
            Self::slide_up,
            Self::slide_down,
            Self::slide_left,
            Self::slide_right,
        ] {
            let mut board = self.clone();
            slide(&mut board);
            if board != *self {
                return false;
            }
        }
        true
    }

    fn show(&self) -> StyledString {
        let line = join(
            color_string(" ".to_owned(), BORDER_COLOR, BORDER_COLOR),
            vec![
                color_string("──────".to_owned(), BORDER_COLOR, BORDER_COLOR);
                self.size
            ],
        );

        let mut s = StyledString::new();
        let mut first = true;
        s.append(line.clone());
        for row in self.rows.iter() {
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
}

#[derive(Debug, PartialEq)]
struct GameState {
    board: Board,
    score: u32,
}

impl GameState {
    fn slide_right(&mut self) {
        let score = self.board.slide_right();
        self.score += score;
    }

    fn slide_left(&mut self) {
        let score = self.board.slide_left();
        self.score += score;
    }

    fn slide_up(&mut self) {
        let score = self.board.slide_up();
        self.score += score;
    }

    fn slide_down(&mut self) {
        let score = self.board.slide_down();
        self.score += score;
    }

    fn add_random_tile(&mut self) {
        self.board.add_random_tile();
    }

    fn is_complete(&self) -> bool {
        self.board.is_complete()
    }

    fn is_stuck(&self) -> bool {
        self.board.is_stuck()
    }

    fn init() -> Self {
        let mut state = GameState {
            board: Board::empty(4),
            score: 0,
        };
        state.add_random_tile();
        state.add_random_tile();
        state
    }

    fn show(&self) -> StyledString {
        let mut s = StyledString::new();
        s.append(self.board.show());
        s.append_plain(format!("Score: {:4}", self.score));
        s
    }
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

fn show_row(row: &Vec<Tile>) -> StyledString {
    let colors: Vec<Color> = row.iter().map(|tile| tile.background_color()).collect();
    let top = join(
        color_string(" ".to_owned(), BORDER_COLOR, BORDER_COLOR),
        colors
            .into_iter()
            .map(|color| color_string("      ".to_owned(), color, color))
            .collect(),
    );
    let mid = join(
        color_string(" ".to_owned(), BORDER_COLOR, BORDER_COLOR),
        row.into_iter().map(|tile| tile.show()).collect(),
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

fn slide_row_right(row: &Vec<Tile>) -> (Vec<Tile>, u32) {
    let n = row.len();
    let numbers: Vec<u32> = row
        .into_iter()
        .filter_map(|tile| match *tile {
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

fn choose<T>(xs: &Vec<T>) -> Option<&T> {
    if xs.len() == 0 {
        return None;
    }
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let i = rng.gen_range(0, xs.len());
    xs.get(i)
}
