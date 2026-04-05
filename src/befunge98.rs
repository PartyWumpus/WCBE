use std::cmp;

use coarsetime::{Duration, Instant};
use egui::ahash::{HashSet, HashSetExt};
use rand::Rng;

use egui::ahash::HashMap;

use crate::{
    app::{self, Settings},
    befunge::{
        Befunge, FungeSpaceTrait, GraphicalEvent, Graphics, Position, StepStatus, Value, Visited,
        WhereVisited,
    },
};

#[derive(Clone)]
pub struct FungeSpace {
    map: HashMap<Position, Value>,
    zero_page: Box<[Value; 100]>,
    max_size: (i64, i64),
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct Direction(Value, Value);

impl Direction {
    const North: Self = Self(0, -1);
    const South: Self = Self(0, 1);
    const East: Self = Self(1, 0);
    const West: Self = Self(-1, 0);

    pub fn reverse(&self) -> Self {
        Self(-self.0, -self.1)
    }

    pub fn turn_right(&self) -> Self {
        Self(-self.1, self.0)
    }

    pub fn turn_left(&self) -> Self {
        Self(self.1, -self.0)
    }
}

#[derive(Clone, Default)]
pub struct State {
    state: StateTempName,
    cursor: Cursor,
}

#[derive(Clone)]
pub struct StateTempName {
    pub instruction_count: usize,
    pub map: FungeSpace,

    pub pos_history: HashMap<Position, Visited>,
    pub get_history: HashMap<Position, Instant>,
    pub put_history: HashMap<Position, Instant>,
    pub output: String,
    pub graphics: Option<Graphics>,
    pub breakpoints: HashSet<Position>,
    //pub input_buffer: VecDeque<i64>,
    pub input_buffer: String,
}

#[derive(Clone)]
pub struct Cursor {
    pub stack: Vec<Value>,
    pub position: Position,
    pub direction: Direction,

    // consider combining into enum
    pub string_mode: bool,
    pub semicolon_mode: bool,
}

impl FungeSpaceTrait for FungeSpace {
    fn set(&mut self, pos: Position, val: Value) {
        if pos.0 < 0 || pos.1 < 0 {
            return;
        };

        self.set_inner(pos, val);
    }

    fn get(&self, pos: Position) -> Value {
        if pos.0 < 0 || pos.1 < 0 {
            return 0;
        }
        if pos.0 < 10 && pos.1 < 10 {
            self.zero_page[(pos.0 + pos.1 * 10) as usize]
        } else {
            *self.map.get(&pos).unwrap_or(&(b' ' as Value))
        }
    }

    fn entries(&self) -> impl Iterator<Item = (Position, Value)> {
        self.map
            .iter()
            .map(|(k, v)| (*k, *v))
            .chain(self.zero_page.iter().enumerate().map(|(i, val)| {
                let i = i as i64;
                ((i % 10, i / 10), *val)
            }))
    }

    fn program_size(&self) -> (i64, i64) {
        self.max_size
    }
}

impl FungeSpace {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            zero_page: Box::new([b' '.into(); 100]),
            max_size: (11, 11),
        }
    }

    pub fn new_from_fungespace(mut input: app::FungeSpace) -> Self {
        let mut zero_page = Box::new([b' '.into(); 100]);
        let max_size = input.program_size();
        for idx in 0..100 {
            if let Some(val) = input.map.remove(&(idx % 10, idx / 10)) {
                zero_page[idx as usize] = val;
            }
        }
        Self {
            map: input.map,
            zero_page,
            max_size,
        }
    }

    fn set_inner(&mut self, pos: Position, val: Value) {
        if pos.0 < 0 || pos.1 < 0 {
            return;
        };

        if pos.0 < 10 && pos.1 < 10 {
            self.zero_page[(pos.0 + pos.1 * 10) as usize] = val
        } else {
            if val == b' ' as Value {
                self.map.remove(&pos);
            } else {
                self.map.insert(pos, val);
            }

            if pos.0 > self.max_size.0 {
                self.max_size.0 = pos.0
            }
            if pos.1 > self.max_size.1 {
                self.max_size.1 = pos.1
            }
        };
    }

    pub fn get_nullable(&self, pos: Position) -> Option<Value> {
        if pos.0 < 10 && pos.1 < 10 {
            Some(self.zero_page[usize::try_from(pos.0 + pos.1 * 10).unwrap()])
        } else {
            self.map.get(&pos).copied()
        }
    }
}

impl Default for StateTempName {
    fn default() -> Self {
        Self {
            instruction_count: 0,
            map: FungeSpace::new(),

            pos_history: HashMap::default(),
            put_history: HashMap::default(),
            get_history: HashMap::default(),
            output: String::new(),
            graphics: None,
            breakpoints: HashSet::new(),
            //input_buffer: VecDeque::new(),
            input_buffer: String::new(),
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            string_mode: false,
            semicolon_mode: false,
            position: (0, 0),
            direction: Direction::East,
            stack: Vec::new(),
        }
    }
}

impl StateTempName {
    pub fn new_from_fungespace(fungespace: app::FungeSpace) -> Self {
        Self {
            map: FungeSpace::new_from_fungespace(fungespace),
            ..Default::default()
        }
    }
}

impl State {
    pub fn new_from_fungespace(fungespace: app::FungeSpace) -> Self {
        Self {
            state: StateTempName::new_from_fungespace(fungespace),
            ..Default::default()
        }
    }

    pub fn step_position(&mut self, settings: &Settings) {
        self.cursor.step_position(&mut self.state, settings);
    }

    pub fn reflect(&mut self) {
        self.cursor.direction = self.cursor.direction.reverse();
    }

    pub fn step(&mut self, settings: &Settings) -> StepStatus {
        self.cursor.step(&mut self.state, settings)
    }
}

impl Cursor {
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(0)
    }

    pub fn step_position(&mut self, state: &mut StateTempName, settings: &Settings) {
        let (x, y) = self.position;
        self.step_position_inner(state);
        if settings.pos_history.0 {
            if let Some(visited) = state.pos_history.get_mut(&(x, y)) {
                match self.direction {
                    Direction::North => {
                        visited.wawa.set_north(true);
                        visited.north = Instant::recent();
                    }
                    Direction::South => {
                        visited.wawa.set_south(true);
                        visited.south = Instant::recent();
                    }
                    Direction::East => {
                        visited.wawa.set_east(true);
                        visited.east = Instant::recent();
                    }
                    Direction::West => {
                        visited.wawa.set_west(true);
                        visited.west = Instant::recent();
                    }
                    _ => {
                        // TODO:
                        visited.wawa.set_east(true);
                        visited.west = Instant::recent();
                    }
                }
            } else {
                state.pos_history.insert(
                    (x, y),
                    match self.direction {
                        Direction::North => Visited {
                            wawa: WhereVisited::new().with_north(true),
                            north: Instant::recent(),
                            ..Default::default()
                        },
                        Direction::South => Visited {
                            wawa: WhereVisited::new().with_south(true),
                            south: Instant::recent(),
                            ..Default::default()
                        },
                        Direction::East => Visited {
                            wawa: WhereVisited::new().with_east(true),
                            east: Instant::recent(),
                            ..Default::default()
                        },
                        Direction::West => Visited {
                            wawa: WhereVisited::new().with_west(true),
                            west: Instant::recent(),
                            ..Default::default()
                        },
                        // TODO:
                        _ => Visited {
                            wawa: WhereVisited::new().with_east(true),
                            east: Instant::recent(),
                            ..Default::default()
                        },
                    },
                );
            }
        }
    }

    fn step_position_inner(&mut self, state: &StateTempName) {
        let (x, y) = self.position;
        self.position = (x + self.direction.0, y + self.direction.1);

        if self.position.0 == -1 {
            self.position.0 = state.map.max_size.0.saturating_add(1);
        } else if self.position.0.wrapping_sub(1) >= state.map.max_size.0 {
            self.position.0 = 0
        };

        if self.position.1 == -1 {
            self.position.1 = state.map.max_size.1.saturating_add(1);
        } else if self.position.1.wrapping_sub(1) >= state.map.max_size.1 {
            self.position.1 = 0
        };
    }

    pub fn step(&mut self, state: &mut StateTempName, settings: &Settings) -> StepStatus {
        state.instruction_count += 1;
        let status = self.step_inner(state, settings);
        if state.breakpoints.contains(&self.position) {
            return StepStatus::Breakpoint;
        }
        if settings.skip_spaces {
            let mut pos = None;
            loop {
                if state.map.get(self.position) == b' ' as Value {
                    pos = Some(self.position);
                    // small visual bug because it steps onto the last space
                    // add a peek position
                    self.step_position(state, settings);
                } else {
                    break;
                }
            }
            if let Some(pos) = pos {
                self.position = pos;
            }
        };
        status
    }

    fn step_inner(&mut self, state: &mut StateTempName, settings: &Settings) -> StepStatus {
        let op = state.map.get_nullable(self.position);

        if self.string_mode {
            let op = op.unwrap_or(b' ' as Value);
            if op == b'"' as Value {
                self.string_mode = false;
            } else {
                self.stack.push(op);
            }
            self.step_position(state, settings);
            StepStatus::Normal
        } else if self.semicolon_mode {
            let op = op.unwrap_or(b' ' as Value);
            if op == b';' as Value {
                self.semicolon_mode = false;
            }
            self.step_position(state, settings);
            StepStatus::Normal
        } else if let Some(op) = op {
            if let Ok(op) = op.try_into() {
                let status = self.do_op(op, state, settings);
                match status {
                    StepStatus::Normal | StepStatus::SyncFrame => {
                        self.step_position(state, settings);
                    }
                    _ => (),
                };
                status
            } else {
                StepStatus::Error("Invalid operation")
            }
        } else {
            self.step_position(state, settings);
            StepStatus::Normal
        }
    }

    fn do_op(&mut self, op: u8, state: &mut StateTempName, settings: &Settings) -> StepStatus {
        match op {
            b'"' => self.string_mode = true,

            b'0'..=b'9' => self.stack.push((op - b'0').into()),

            // 2 op operations
            b'+' => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(b + a);
            }
            b'-' => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(b - a);
            }
            b'*' => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(b * a);
            }
            b'/' => {
                let a = self.pop();
                let b = self.pop();
                if a == 0 {
                    self.stack.push(0);
                } else {
                    self.stack.push(b / a);
                }
            }
            b'%' => {
                let a = self.pop();
                let b = self.pop();
                if a == 0 {
                    self.stack.push(0);
                } else {
                    self.stack.push(b % a);
                }
            }
            b'`' => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(if b > a { 1 } else { 0 });
            }
            b'\\' => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(a);
                self.stack.push(b);
            }

            // one op operations
            b'!' => {
                let a = self.pop();
                self.stack.push(if a == 0 { 1 } else { 0 });
            }
            b':' => {
                let a = self.pop();
                self.stack.push(a);
                self.stack.push(a);
            }
            b'$' => {
                self.pop();
            }

            // static direction changes
            b'>' => self.direction = Direction::East,
            b'<' => self.direction = Direction::West,
            b'^' => self.direction = Direction::North,
            b'v' => self.direction = Direction::South,
            b'#' => {
                self.step_position(state, settings);
                self.step_position_inner(state);
                return StepStatus::NormalNoStep;
            }

            // dynamic direction changes
            // FIXME:
            //b'?' => self.direction = rand::thread_rng().r#gen(),
            b'_' => {
                let status = self.pop();
                if status == 0 {
                    self.direction = Direction::East;
                } else {
                    self.direction = Direction::West;
                }
            }

            b'|' => {
                let status = self.pop();
                if status == 0 {
                    self.direction = Direction::South;
                } else {
                    self.direction = Direction::North;
                }
            }

            // put (this is the big one!)
            b'p' => {
                let y = self.pop();
                let x = self.pop();
                let value = self.pop();

                if settings.put_history.0 {
                    if let Some(prev_time) = state.put_history.get(&(x, y)) {
                        if prev_time.elapsed_since_recent() > Duration::from_millis(500) {
                            state.put_history.insert((x, y), Instant::recent());
                        }
                    } else {
                        state.put_history.insert((x, y), Instant::recent());
                    }
                }

                state.map.set((x, y), value);
            }

            // get
            b'g' => {
                let y = self.pop();
                let x = self.pop();
                self.stack.push(state.map.get((x, y)));

                if settings.get_history.0 {
                    if let Some(prev_time) = state.get_history.get(&(x, y)) {
                        if prev_time.elapsed_since_recent() > Duration::from_millis(500) {
                            state.get_history.insert((x, y), Instant::recent());
                        }
                    } else {
                        state.get_history.insert((x, y), Instant::recent());
                    }
                }
            }

            // input
            b'&' => {
                let mut itr = state.input_buffer.chars();
                let mut num = 0;
                loop {
                    match itr.next() {
                        None => {
                            if settings.non_blocking_input {
                                self.stack.push(-1);
                                return StepStatus::Normal;
                            } else {
                                return StepStatus::Breakpoint;
                            }
                        }
                        Some(val @ '0'..='9') => {
                            num *= 10;
                            num += (val as u8 - b'0') as Value;
                        }
                        Some(' ') => {
                            self.stack.push(num);
                            state.input_buffer = itr.as_str().into();
                            return StepStatus::Normal;
                        }
                        Some(_) => {
                            return StepStatus::Error("Invalid input for Error::InvalidNumber");
                        }
                    }
                }
            }

            b'~' => {
                let mut itr = state.input_buffer.chars();
                match itr.next() {
                    None => {
                        if settings.non_blocking_input {
                            self.stack.push(-1);
                        } else {
                            return StepStatus::Breakpoint;
                        }
                    }
                    Some(chr) => {
                        self.stack.push(chr as Value);
                        state.input_buffer = itr.as_str().into();
                    }
                }
            }

            // halt is dealt with higher up
            b'@' => return StepStatus::Breakpoint,

            // -- IO output
            b'.' => {
                let a = self.pop().to_string();
                state.output.push_str(&a);
                state.output.push(' ');
            }
            b',' => {
                let Ok(a) = (self.pop() as u32).try_into() else {
                    return StepStatus::Error("Invalid UTF-8 char");
                };
                state.output.push(a);
            }

            b'\'' => {
                self.step_position(state, settings);
                self.stack.push(state.map.get(self.position));
                self.step_position_inner(state);
                return StepStatus::NormalNoStep;
            }

            b';' => {
                self.semicolon_mode = true;
            }

            b'[' => {
                self.direction = self.direction.turn_left();
            }

            b']' => {
                self.direction = self.direction.turn_right();
            }

            b'=' => return StepStatus::Error("Execute is not yet implemented"),

            b'A'..=b'Z' | b'(' | b')' => {
                return StepStatus::Error("Fingerprints are not yet implemented");
            }

            b'a'..=b'f' => self.stack.push((op - b'a' + 10).into()),

            b'j' => {
                let count = self.pop();

                if count < 0 {
                    self.direction = self.direction.reverse();
                    for _ in count..-1 {
                        self.step_position_inner(state);
                    }
                    self.direction = self.direction.reverse();
                    return StepStatus::NormalNoStep;
                } else {
                    for _ in 0..count {
                        self.step_position_inner(state);
                    }
                }
            }
            b'k' => {
                let count = self.pop();

                if count == 0 {
                    self.step_position(state, settings);
                    self.step_position_inner(state);
                    return StepStatus::NormalNoStep;
                }

                let (x, y) = self.position;
                // TODO: loop detection
                let mut op;
                loop {
                    self.step_position_inner(state);
                    op = state.map.get(self.position);
                    if op != b' ' as Value {
                        break;
                    }
                }
                self.position = (x, y);

                if let Ok(op) = op.try_into() {
                    for _ in 0..count {
                        match op {
                            b'#' => {
                                self.step_position_inner(state);
                            }
                            _ => {
                                self.do_op(op, state, settings);
                            }
                        };
                    }
                }
                return StepStatus::Normal;
            }

            b'n' => {
                self.stack.clear();
            }

            b'q' => return StepStatus::Error("Quit is not yet properly implemented"),

            b'r' => {
                self.direction = self.direction.reverse();
            }

            b's' => {
                self.step_position(state, settings);
                let a = self.pop();
                state.map.set_inner(self.position, a);
                self.step_position_inner(state);
                return StepStatus::NormalNoStep;
            }

            b't' => return StepStatus::Error("Split is not yet implemented"),

            b'u' => return StepStatus::Error("Stack-under-stack is not yet implemented"),

            b'w' => {
                let a = self.pop();
                let b = self.pop();
                match a.cmp(&b) {
                    cmp::Ordering::Greater => self.direction = self.direction.turn_left(),

                    cmp::Ordering::Less => self.direction = self.direction.turn_right(),
                    cmp::Ordering::Equal => (),
                }
            }

            b'y' => return StepStatus::Error("Sysinfo is not yet implemented"),

            b'{' | b'}' => return StepStatus::Error("Blocks are not yet implemented"),

            b'x' => {
                let y = self.pop();
                let x = self.pop();
                self.direction = Direction(x, y);
            }

            // noop
            b' ' | b'z' => (),

            _ => return StepStatus::Error("Invalid operation"),
        };
        StepStatus::Normal
    }
}

impl Befunge for State {
    fn get(&self, pos: Position) -> Value {
        self.state.map.get(pos)
    }
    fn set(&mut self, pos: Position, val: Value) {
        self.state.map.set(pos, val);
    }
    fn step(&mut self, settings: &Settings) -> StepStatus {
        self.step(settings)
    }

    fn program_size(&self) -> Position {
        self.state.map.max_size
    }
    fn instruction_count(&self) -> usize {
        self.state.instruction_count
    }
    fn string_mode(&self) -> bool {
        self.cursor.string_mode
    }
    fn cursor_position(&self) -> Position {
        self.cursor.position
    }
    fn cursor_direction(&self) -> (Value, Value) {
        (self.cursor.direction.0, self.cursor.direction.1)
    }

    fn stack(&self) -> Vec<Value> {
        self.cursor.stack.clone()
    }
    fn stdout(&self) -> &str {
        &self.state.output
    }
    fn stdin(&mut self) -> &mut String {
        &mut self.state.input_buffer
    }
    fn graphics(&mut self) -> Option<&mut Graphics> {
        self.state.graphics.as_mut()
    }

    fn pos_history(&mut self) -> &mut HashMap<Position, Visited> {
        &mut self.state.pos_history
    }
    fn get_history(&mut self) -> &mut HashMap<Position, Instant> {
        &mut self.state.get_history
    }
    fn put_history(&mut self) -> &mut HashMap<Position, Instant> {
        &mut self.state.put_history
    }
    fn breakpoints(&mut self) -> &mut HashSet<Position> {
        &mut self.state.breakpoints
    }

    fn serialize(&self) -> String {
        self.state.map.serialize()
    }
}
