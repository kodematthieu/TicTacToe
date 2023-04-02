use std::ops::Deref;

use druid::Data;

#[derive(Clone, Data, Debug)]
pub struct TicTacToe {
    cells: [State; 9],
    state: State,
    done: Option<u8>
}
impl TicTacToe {
    pub fn new(mut first: State) -> Self {
        first = match first {
            State::N if rand::random() => State::X,
            State::N => State::O,
            x => x
        };

        Self {
            cells: [State::N; 9],
            state: first,
            done: None
        }
    }
    #[inline]
    pub fn done(&self) -> Option<u8> {
        self.done
    }
    #[inline]
    pub fn state(&self) -> State {
        self.state
    }
    pub fn set(&mut self, idx: usize) -> bool {
        if self.state != State::N {
            match self.cells.get_mut(idx) {
                Some(x @ &mut State::N) => {
                    *x = self.state;
                    if let Some((state, orien)) = self.calc(idx) {
                        self.done = Some(orien);
                        self.state = state;
                    } else  {
                        self.state.invert();
                    }

                    true
                },
                _ => false
            }
        } else {false}
    }
    pub fn get(&self, idx: usize) -> State {
        *self.cells.get(idx).unwrap_or(&State::N)
    }
    pub fn draw(&self) -> bool {
        self.done.is_none() && self.cells.iter().all(|x| x != &State::N)
    }
    pub fn calc(&self, idx: usize) -> Option<(State, u8)> {
        self.calc_row(Self::row_of(idx))
            .or_else(|| self.calc_col(Self::col_of(idx)))
            .or_else(|| match idx {
                4 => self.calc_dia(false).or_else(|| self.calc_dia(true)),
                x if x % 4 == 0 => self.calc_dia(false),
                x if x != 8 && x != 0 && x % 2 == 0 => self.calc_dia(true),
                _ => None
            })
    }
    fn calc_row(&self, row: usize) -> Option<(State, u8)> {
        let idx = row * 3;
        let state = self.cells.get(idx)?;
        let state2 = self.cells.get(idx + 1)?;
        if state == state2 && state2 == self.cells.get(idx + 2)? {
            Some((*state, row as _))
        } else {
            None
        }
    }
    fn calc_col(&self, col: usize) -> Option<(State, u8)> {
        let state = self.cells.get(col)?;
        let state2 = self.cells.get(3 + col)?;
        if state == state2 && state2 == self.cells.get(6 + col)? {
            Some((*state, col as u8 + 3))
        } else {
            None
        }
    }
    fn calc_dia(&self, right: bool) -> Option<(State, u8)> {
        if right {
            let state = self.cells.get(2)?;
            let state2 = self.cells.get(4)?;
            if state == state2 && state2 == self.cells.get(6)? {
                Some((*state, 7))
            } else {
                None
            }
        } else {
            let state = self.cells.get(0)?;
            let state2 = self.cells.get(4)?;
            if state == state2 && state2 == self.cells.get(8)? {
                Some((*state, 6))
            } else {
                None
            }
        }
    }
    #[inline]
    fn row_of(idx: usize) -> usize {
        // 0 1 2 3 4 5 6 7 8
        //   0     1     2
        idx / 3
    }
    #[inline]
    fn col_of(idx: usize) -> usize {
        // 0 3 6 1 4 7 2 5 8
        //   0     1     2
        idx % 3
    }
}
impl Deref for TicTacToe {
    type Target = [State; 9];
    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

#[derive(Clone, Copy, Data, Debug, Default, Eq, PartialEq)]
pub enum State {X, O, #[default] N}
impl State {
    #[inline]
    fn invert(&mut self) {
        *self = match self {
            State::X => State::O,
            State::O => State::X,
            State::N => State::N,
        }
    }
}