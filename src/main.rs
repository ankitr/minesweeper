mod error;

use error::GameError;
use rand::{seq::IteratorRandom, thread_rng};
use std::char;
use std::fmt;
use std::io;
use std::ops::Index;

// const MAXIMUM_BOMBS: usize = 100;
// const MAXIMUM_DIMENSION: usize = 255;

#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy)]
enum SquareState {
    Covered,
    Uncovered,
    Bomb,
}

#[derive(Debug)]
struct Board {
    squares: Vec<SquareState>,
    height: usize,
    width: usize,
}

impl Index<(usize, usize)> for Board {
    type Output = SquareState;
    fn index(&self, index: (usize, usize)) -> &SquareState {
        &self.squares[index.0 * &self.height + index.1]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        for index in 0..(self.height * self.width) {
            if index % self.height == 0 {
                output.push('\n');
            }
            output.push(match self.squares[index] {
                SquareState::Uncovered => {
                    let surrounding_bomb_count = self.surrounding_bombs_from_index(index);
                    if surrounding_bomb_count == 0 {
                        ' '
                    } else {
                        match char::from_digit(surrounding_bomb_count as u32, 10) {
                            Some(c) => c,
                            None => panic!("Found >9 bombs around a square?"), // Won't happen, as there can be at most 8 bombs around you
                        }
                    }
                }
                SquareState::Bomb | SquareState::Covered => 'x',
            });
            output.push(' ');
        }
        write!(f, "{}", output)
    }
}

fn surrounding_squares(position: Position, width: usize, height: usize) -> Vec<Position> {
    let mut output = vec![position];
    if position.x > 0 {
        output.push(Position {
            x: position.x - 1,
            y: position.y,
        });

        if position.y > 0 {
            output.push(Position {
                x: position.x - 1,
                y: position.y - 1,
            });
        }
        if position.y < height - 1 {
            output.push(Position {
                x: position.x - 1,
                y: position.y + 1,
            });
        }
    }
    if position.x < width - 1 {
        output.push(Position {
            x: position.x + 1,
            y: position.y,
        });
        if position.y > 0 {
            output.push(Position {
                x: position.x + 1,
                y: position.y - 1,
            });
        }
        if position.y < height - 1 {
            output.push(Position {
                x: position.x + 1,
                y: position.y + 1,
            });
        }
    }

    if position.y > 0 {
        output.push(Position {
            x: position.x,
            y: position.y - 1,
        });
    }
    if position.y < height - 1 {
        output.push(Position {
            x: position.x,
            y: position.y + 1,
        });
    }
    output
}

impl Board {
    fn surrounding_squares_positions(&self, position: Position) -> Vec<Position> {
        surrounding_squares(position, self.width, self.height)
    }

    fn surrounding_squares_indexes(&self, index: usize) -> Vec<usize> {
        self.surrounding_squares_positions(self.position_from_index(index))
            .iter()
            .map(|position: &Position| self.index_from_position(position))
            .collect()
    }

    fn index_from_position(&self, position: &Position) -> usize {
        (position.y * self.height) + position.x
    }

    fn position_from_index(&self, index: usize) -> Position {
        Position {
            x: index % self.height,
            y: index / self.height,
        }
    }

    fn surrounding_bombs_from_index(&self, index: usize) -> usize {
        self.surrounding_squares_indexes(index)
            .iter()
            .filter(|surrounding_index| {
                matches!(self.squares[**surrounding_index], SquareState::Bomb)
            })
            .count()
    }
    fn contains(&self, position: &Position) -> bool {
        (0..self.width).contains(&position.x) && (0..self.height).contains(&position.y)
    }
}

#[derive(Debug)]
struct Game {
    bomb_count: usize,
    board: Board,
    move_count: u8,
}

impl fmt::Display for Game {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.board)
    }
}

impl Game {
    fn start(
        width: usize,
        height: usize,
        bomb_count: usize,
        start_move: Position,
    ) -> Result<Game, GameError> {
        if !(0..height).contains(&start_move.x) | !(0..width).contains(&start_move.y) {
            return Err(GameError::OutOfBoundsError {
                x: start_move.x,
                y: start_move.y,
            });
        }

        // First, we check find the number of guaranteed clear squares
        enum SquarePosition {
            Corner,
            Edge,
            Middle,
        }
        let cleared_square_position: SquarePosition = {
            let on_height_edge = !(1..(height - 1)).contains(&start_move.x);
            let on_width_edge = !(1..(width - 1)).contains(&start_move.y);
            // TODO: handle the 1 x 1 case
            if on_height_edge & on_width_edge {
                SquarePosition::Corner
            } else if on_height_edge | on_width_edge {
                SquarePosition::Edge
            } else {
                SquarePosition::Middle
            }
        };

        let position_to_index = |position: &Position| position.y * height + position.x;

        let cleared = surrounding_squares(start_move, width, height);

        let mut rng = thread_rng();
        let index_range: Vec<usize> = (0..((height * width)
            - match cleared_square_position {
                SquarePosition::Corner => 4,
                SquarePosition::Edge => 6,
                SquarePosition::Middle => 9,
            }))
            .collect();
        let unadjusted_bomb_position_indexes =
            index_range.iter().choose_multiple(&mut rng, bomb_count);
        let mut cleared_indexes: Vec<usize> = cleared.iter().map(position_to_index).collect();
        cleared_indexes.sort();
        let adjusted_bomb_position_indexes: Vec<usize> = unadjusted_bomb_position_indexes
            .iter()
            .map(|bomb_index: &&usize| {
                let adjustment = cleared_indexes
                    .iter()
                    .filter(|cleared_index: &&usize| bomb_index >= cleared_index)
                    .count();
                *bomb_index + adjustment
            })
            .collect();

        let mut new_game = Game {
            bomb_count: bomb_count,
            board: Board {
                squares: (0..(width * height))
                    .map(|index: usize| {
                        if cleared_indexes.contains(&index) {
                            SquareState::Uncovered
                        } else if adjusted_bomb_position_indexes.contains(&index) {
                            SquareState::Bomb
                        } else {
                            SquareState::Covered
                        }
                    })
                    .collect(),
                width: width,
                height: height,
            },
            move_count: 1,
        };
        new_game.clear_safe_squares();
        Ok(new_game)
    }

    fn clear_safe_square(&mut self, index: usize) {
        for clear_index in self.board.surrounding_squares_indexes(index) {
            if !matches!(self.board.squares[clear_index], SquareState::Uncovered) {
                self.board.squares[clear_index] = SquareState::Uncovered;
                if self.board.surrounding_bombs_from_index(clear_index) == 0 {
                    self.clear_safe_square(clear_index)
                }
            }
        }
    }

    fn clear_safe_squares(&mut self) {
        for index in 0..(self.board.width * self.board.height) {
            if matches!(self.board.squares[index], SquareState::Uncovered)
                && self.board.surrounding_bombs_from_index(index) == 0
            {
                self.clear_safe_square(index)
            }
        }
    }

    fn get_move_input(&self) -> Result<Position, GameError> {
        println!("{}", self.board);
        println!("Enter your move");
        let mut user_move_input = String::new();
        io::stdin()
            .read_line(&mut user_move_input)
            .expect("Failed to read line");
        let user_move_str_components: Vec<&str> = user_move_input.split(',').collect();
        if user_move_str_components.len() != 2 {
            return Err(GameError::InvalidMoveError);
        };
        Ok(Position {
            x: user_move_str_components[0].trim().parse().unwrap(),
            y: user_move_str_components[1].trim().parse().unwrap(),
        })
    }

    fn check_won(&self) -> bool {
        self.board
            .squares
            .iter()
            .all(|square_state| !matches!(square_state, SquareState::Uncovered))
    }

    fn make_move_io(&mut self) -> Result<(), GameError> {
        let target_position = self.get_move_input()?;
        if !self.board.contains(&target_position) {
            return Err(GameError::InvalidMoveError);
        };

        let move_square_index = self.board.index_from_position(&target_position);
        let move_square_state = self.board.squares[move_square_index];
        self.move_count += 1;
        match move_square_state {
            SquareState::Bomb => {
                println!("you died");
                Ok(())
            }
            SquareState::Uncovered => Err(GameError::RepeatMoveError),
            SquareState::Covered => {
                self.board.squares[move_square_index] = SquareState::Uncovered;
                self.clear_safe_squares();
                if self.check_won() {
                    println!("you won");
                    Ok(())
                } else {
                    self.make_move_io()
                }
            }
        }
    }
}

fn main() {
    let mut game = Game::start(10, 10, 10, Position { x: 5, y: 5 }).unwrap();
    game.make_move_io().unwrap();
}
