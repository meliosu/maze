use std::{io::Write, time::Instant};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, ContentStyle, Print, PrintStyledContent, StyledContent},
    terminal::{
        self, BeginSynchronizedUpdate, EndSynchronizedUpdate, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use rand::{seq::SliceRandom, Rng};

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _ = terminal::disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        println!("thread {info}");
    }));

    let mut stdout = std::io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let (cols, rows) = {
        let (cols, rows) = terminal::size()?;
        (cols - 1 | 1, rows - 1 | 1)
    };

    let maze = Maze::random_perfect(cols as usize, rows as usize);

    let (end_x, end_y) = {
        let mut rng = rand::thread_rng();

        (
            rng.gen_range(0..cols / 2) * 2 + 1,
            rng.gen_range(0..rows / 2) * 2 + 1,
        )
    };

    render_maze(&maze)?;

    render_end(end_x, end_y)?;

    let (mut x, mut y) = (1, 1);
    let start = Instant::now();

    let (won, elapsed) = loop {
        if x == end_x && y == end_y {
            break (true, start.elapsed());
        }

        render_player(x, y)?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l'),
                ..
            }) if x < cols - 1 => {
                if maze.cells[y as usize][x as usize + 1] != Cell::Wall {
                    execute!(stdout, MoveTo(x, y), Print(" "))?;
                    x += 1;
                }
            }

            Event::Key(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k'),
                ..
            }) if y > 0 => {
                if maze.cells[y as usize - 1][x as usize] != Cell::Wall {
                    execute!(stdout, MoveTo(x, y), Print(" "))?;
                    y -= 1;
                }
            }

            Event::Key(KeyEvent {
                code: KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h'),
                ..
            }) if x > 0 => {
                if maze.cells[y as usize][x as usize - 1] != Cell::Wall {
                    execute!(stdout, MoveTo(x, y), Print(" "))?;
                    x -= 1;
                }
            }

            Event::Key(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j'),
                ..
            }) if y < rows - 1 => {
                if maze.cells[y as usize + 1][x as usize] != Cell::Wall {
                    execute!(stdout, MoveTo(x, y), Print(" "))?;
                    y += 1;
                }
            }

            Event::Key(KeyEvent {
                code: KeyCode::Esc | KeyCode::Char('q'),
                ..
            }) => break (false, start.elapsed()),

            _ => {}
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;

    if won {
        println!("🦀🦀🦀Congrats🦀🦀🦀!!! You are a winner in life!");
        println!("It took {elapsed:?} to beat the maze...");
    } else {
        println!("You gave up after {elapsed:?} :(");
        println!("Maybe try again later...");
    }

    Ok(())
}

fn render_end(x: u16, y: u16) -> std::io::Result<()> {
    let style_red = ContentStyle {
        foreground_color: Some(Color::Red),
        ..Default::default()
    };

    let end_square = StyledContent::new(style_red, "■");

    execute!(
        std::io::stdout(),
        MoveTo(x, y),
        PrintStyledContent(end_square)
    )
}

fn render_player(x: u16, y: u16) -> std::io::Result<()> {
    let style_blue = ContentStyle {
        foreground_color: Some(Color::Blue),
        ..Default::default()
    };

    let player = StyledContent::new(style_blue, "∲");

    execute!(std::io::stdout(), MoveTo(x, y), PrintStyledContent(player))
}

fn render_maze(maze: &Maze) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();

    execute!(stdout, BeginSynchronizedUpdate)?;

    for i in 0..maze.width {
        for j in 0..maze.height {
            let right = if i < maze.width - 1 {
                maze.cells[j][i + 1]
            } else {
                Cell::Path
            };

            let down = if j < maze.height - 1 {
                maze.cells[j + 1][i]
            } else {
                Cell::Path
            };

            let up = if j > 0 {
                maze.cells[j - 1][i]
            } else {
                Cell::Path
            };

            let left = if i > 0 {
                maze.cells[j][i - 1]
            } else {
                Cell::Path
            };

            use Cell::{Path as P, Wall as W};

            let symbol = match maze.cells[j][i] {
                Cell::Wall => match (right, up, left, down) {
                    (W, W, W, W) => "╬",
                    (W, W, W, P) => "╩",
                    (W, W, P, W) => "╠",
                    (W, W, P, P) => "╚",
                    (W, P, W, W) => "╦",
                    (W, P, W, P) => "═",
                    (W, P, P, W) => "╔",
                    (W, P, P, P) => "═",
                    (P, W, W, W) => "╣",
                    (P, W, W, P) => "╝",
                    (P, W, P, W) => "║",
                    (P, W, P, P) => "║",
                    (P, P, W, W) => "╗",
                    (P, P, W, P) => "═",
                    (P, P, P, W) => "║",
                    (P, P, P, P) => "·",
                },

                Cell::Path => " ",
            };

            queue!(stdout, MoveTo(i as u16, j as u16), Print(symbol))?;
        }
    }

    execute!(stdout, EndSynchronizedUpdate)?;

    stdout.flush()?;

    Ok(())
}

impl Maze {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![Cell::Wall; width]; height],
        }
    }

    fn random_perfect(width: usize, height: usize) -> Self {
        let mut maze = Self::new(width, height);

        let (maze_width, maze_height) = (width / 2, height / 2);

        let mut nodes: Vec<Vec<Direction>> = (0..maze_height)
            .map(|_| {
                let mut row: Vec<Direction> =
                    (0..maze_width - 1).map(|_| Direction::Left).collect();
                row.insert(0, Direction::Up);
                row
            })
            .collect();

        let mut rng = rand::thread_rng();
        let (mut rx, mut ry) = (0, 0);

        for _ in 0..width * height * 10 {
            let mut directions = vec![];

            directions.extend((rx > 0).then_some(Direction::Left));
            directions.extend((ry > 0).then_some(Direction::Up));
            directions.extend((rx < maze_width - 1).then_some(Direction::Right));
            directions.extend((ry < maze_height - 1).then_some(Direction::Down));

            let Some(direction) = directions.choose(&mut rng) else {
                panic!("directions vector should not be empty, hmm");
            };

            nodes[ry][rx] = *direction;

            match direction {
                Direction::Up => ry -= 1,
                Direction::Down => ry += 1,
                Direction::Left => rx -= 1,
                Direction::Right => rx += 1,
            }
        }

        for i in 0..maze_width {
            for j in 0..maze_height {
                let (x, y) = (2 * i + 1, 2 * j + 1);

                maze.cells[y][x] = Cell::Path;

                if i == rx && j == ry {
                    continue;
                }

                let (x, y) = match nodes[j][i] {
                    Direction::Up => (x, y - 1),
                    Direction::Down => (x, y + 1),
                    Direction::Left => (x - 1, y),
                    Direction::Right => (x + 1, y),
                };

                maze.cells[y][x] = Cell::Path;
            }
        }

        maze
    }
}

struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Cell {
    Wall,
    Path,
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
