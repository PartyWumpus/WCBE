use std::cmp;

use coarsetime::{Duration, Instant};
use egui::{
    Color32,
    ahash::{HashSet, HashSetExt},
};
use rand::Rng;

use egui::ahash::HashMap;

use crate::{
    app::{self, Settings},
    befunge::{
        Befunge, Direction, FungeSpaceTrait, GraphicalEvent, Graphics, Position, StepStatus, Value,
        Visited, WhereVisited,
    },
};

#[derive(Clone)]
pub struct FungeSpace {
    map: HashMap<Position, Value>,
    zero_page: Box<[Value; 100]>,
    max_size: (i64, i64),
}

#[derive(Clone)]
pub struct State {
    pub instruction_count: usize,
    pub map: FungeSpace,

    // consider combining into enum
    pub string_mode: bool,
    pub semicolon_mode: bool,

    pub position: Position,
    pub direction: Direction,
    pub pos_history: HashMap<Position, Visited>,
    pub get_history: HashMap<Position, Instant>,
    pub put_history: HashMap<Position, Instant>,
    pub stack: Vec<Value>,
    pub output: String,
    pub graphics: Option<Graphics>,
    pub breakpoints: HashSet<Position>,
    //pub input_buffer: VecDeque<i64>,
    pub input_buffer: String,
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

impl Default for State {
    fn default() -> Self {
        Self {
            instruction_count: 0,
            map: FungeSpace::new(),
            string_mode: false,
            semicolon_mode: false,
            position: (0, 0),
            direction: Direction::East,
            pos_history: HashMap::default(),
            put_history: HashMap::default(),
            get_history: HashMap::default(),
            stack: Vec::new(),
            output: String::new(),
            graphics: None,
            breakpoints: HashSet::new(),
            //input_buffer: VecDeque::new(),
            input_buffer: String::new(),
        }
    }
}

impl State {
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(0)
    }

    pub fn new_from_fungespace(fungespace: app::FungeSpace) -> Self {
        Self {
            map: FungeSpace::new_from_fungespace(fungespace),
            ..Default::default()
        }
    }

    pub fn step_position(&mut self, settings: &Settings) {
        let (x, y) = self.position;
        self.step_position_inner();
        if settings.pos_history.0 {
            if let Some(visited) = self.pos_history.get_mut(&(x, y)) {
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
                }
            } else {
                self.pos_history.insert(
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
                    },
                );
            }
        }
    }

    fn step_position_inner(&mut self) {
        let (x, y) = self.position;
        match self.direction {
            Direction::North => self.position = (x, y - 1),
            Direction::South => self.position = (x, y + 1),
            Direction::East => self.position = (x + 1, y),
            Direction::West => self.position = (x - 1, y),
        }

        if self.position.0 == -1 {
            self.position.0 = self.map.max_size.0.saturating_add(1);
        } else if self.position.0.wrapping_sub(1) >= self.map.max_size.0 {
            self.position.0 = 0
        };

        if self.position.1 == -1 {
            self.position.1 = self.map.max_size.1.saturating_add(1);
        } else if self.position.1.wrapping_sub(1) >= self.map.max_size.1 {
            self.position.1 = 0
        };
    }

    pub fn step(&mut self, settings: &Settings) -> StepStatus {
        self.instruction_count += 1;
        let status = self.step_inner(settings);
        if self.breakpoints.contains(&self.position) {
            return StepStatus::Breakpoint;
        }
        if settings.skip_spaces {
            let mut pos = None;
            loop {
                if self.map.get(self.position) == b' ' as Value {
                    pos = Some(self.position);
                    // small visual bug because it steps onto the last space
                    // add a peek position
                    self.step_position(settings);
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

    fn step_inner(&mut self, settings: &Settings) -> StepStatus {
        let op = self.map.get_nullable(self.position);

        if self.string_mode {
            let op = op.unwrap_or(b' ' as Value);
            if op == b'"' as Value {
                self.string_mode = false;
            } else {
                self.stack.push(op);
            }
            self.step_position(settings);
            StepStatus::Normal
        } else if self.semicolon_mode {
            let op = op.unwrap_or(b' ' as Value);
            if op == b';' as Value {
                self.semicolon_mode = false;
            }
            self.step_position(settings);
            StepStatus::Normal
        } else if let Some(op) = op {
            if let Ok(op) = op.try_into() {
                let status = self.do_op(op, settings);
                match status {
                    StepStatus::Normal | StepStatus::SyncFrame => {
                        self.step_position(settings);
                    }
                    _ => (),
                };
                status
            } else {
                StepStatus::Error("Invalid operation")
            }
        } else {
            self.step_position(settings);
            StepStatus::Normal
        }
    }

    fn do_op(&mut self, op: u8, settings: &Settings) -> StepStatus {
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
                self.step_position(settings);
                self.step_position_inner();
                return StepStatus::NormalNoStep;
            }

            // dynamic direction changes
            b'?' => self.direction = rand::thread_rng().r#gen(),
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
                    if let Some(prev_time) = self.put_history.get(&(x, y)) {
                        if prev_time.elapsed_since_recent() > Duration::from_millis(500) {
                            self.put_history.insert((x, y), Instant::recent());
                        }
                    } else {
                        self.put_history.insert((x, y), Instant::recent());
                    }
                }

                self.map.set((x, y), value);
            }

            // get
            b'g' => {
                let y = self.pop();
                let x = self.pop();
                self.stack.push(self.map.get((x, y)));

                if settings.get_history.0 {
                    if let Some(prev_time) = self.get_history.get(&(x, y)) {
                        if prev_time.elapsed_since_recent() > Duration::from_millis(500) {
                            self.get_history.insert((x, y), Instant::recent());
                        }
                    } else {
                        self.get_history.insert((x, y), Instant::recent());
                    }
                }
            }

            // input
            b'&' => {
                let mut itr = self.input_buffer.chars();
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
                            self.input_buffer = itr.as_str().into();
                            return StepStatus::Normal;
                        }
                        Some(_) => {
                            return StepStatus::Error("Invalid input for Error::InvalidNumber");
                        }
                    }
                }
            }

            b'~' => {
                let mut itr = self.input_buffer.chars();
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
                        self.input_buffer = itr.as_str().into();
                    }
                }
            }

            // halt is dealt with higher up
            b'@' => return StepStatus::Breakpoint,

            // -- IO output
            b'.' => {
                let a = self.pop().to_string();
                self.output.push_str(&a);
                self.output.push(' ');
            }
            b',' => {
                let Ok(a) = (self.pop() as u32).try_into() else {
                    return StepStatus::Error("Invalid UTF-8 char");
                };
                self.output.push(a);
            }

            b'\'' => {
                self.step_position(settings);
                self.stack.push(self.map.get(self.position));
                self.step_position_inner();
                return StepStatus::NormalNoStep;
            }

            b';' => {
                self.semicolon_mode = true;
            }

            b'[' => {
                self.direction = match self.direction {
                    Direction::North => Direction::West,
                    Direction::South => Direction::East,
                    Direction::East => Direction::North,
                    Direction::West => Direction::South,
                }
            }

            b']' => {
                self.direction = match self.direction {
                    Direction::North => Direction::East,
                    Direction::South => Direction::West,
                    Direction::East => Direction::South,
                    Direction::West => Direction::North,
                }
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
                        self.step_position_inner();
                    }
                    self.direction = self.direction.reverse();
                    return StepStatus::NormalNoStep;
                } else {
                    for _ in 0..count {
                        self.step_position_inner();
                    }
                }
            }
            b'k' => {
                let count = self.pop();

                if count == 0 {
                    self.step_position(settings);
                    self.step_position_inner();
                    return StepStatus::NormalNoStep;
                }

                let (x, y) = self.position;
                // TODO: loop detection
                let mut op;
                loop {
                    self.step_position_inner();
                    op = self.map.get(self.position);
                    if op != b' ' as Value {
                        break;
                    }
                }
                self.position = (x, y);

                if let Ok(op) = op.try_into() {
                    for _ in 0..count {
                        match op {
                            b'#' => {
                                self.step_position_inner();
                            }
                            _ => {
                                self.do_op(op, settings);
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
                self.direction = match self.direction {
                    Direction::North => Direction::South,
                    Direction::South => Direction::North,
                    Direction::East => Direction::West,
                    Direction::West => Direction::East,
                }
            }

            b's' => {
                self.step_position(settings);
                let a = self.pop();
                self.map.set_inner(self.position, a);
                self.step_position_inner();
                return StepStatus::NormalNoStep;
            }

            b't' => return StepStatus::Error("Split is not yet implemented"),

            b'u' => return StepStatus::Error("Stack-under-stack is not yet implemented"),

            b'w' => {
                let a = self.pop();
                let b = self.pop();
                match a.cmp(&b) {
                    cmp::Ordering::Greater => {
                        self.direction = match self.direction {
                            Direction::North => Direction::West,
                            Direction::South => Direction::East,
                            Direction::East => Direction::North,
                            Direction::West => Direction::South,
                        }
                    }

                    cmp::Ordering::Less => {
                        self.direction = match self.direction {
                            Direction::North => Direction::East,
                            Direction::South => Direction::West,
                            Direction::East => Direction::South,
                            Direction::West => Direction::North,
                        }
                    }
                    cmp::Ordering::Equal => (),
                }
            }

            b'y' => return StepStatus::Error("Sysinfo is not yet implemented"),

            b'{' | b'}' => return StepStatus::Error("Blocks are not yet implemented"),

            b'x' => {
                let y = self.pop();
                let x = self.pop();
                self.direction = match (x, y) {
                    (0, 1) => Direction::North,
                    (1, 0) => Direction::East,
                    (0, -1) => Direction::South,
                    (-1, 0) => Direction::East,
                    _ => return StepStatus::Error("Invalid vector for x"),
                }
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
        self.map.get(pos)
    }
    fn set(&mut self, pos: Position, val: Value) {
        self.map.set(pos, val);
    }
    fn step(&mut self, settings: &Settings) -> StepStatus {
        self.step(settings)
    }

    fn program_size(&self) -> Position {
        self.map.max_size
    }
    fn instruction_count(&self) -> usize {
        self.instruction_count
    }
    fn string_mode(&self) -> bool {
        self.string_mode
    }
    fn cursor_position(&self) -> Position {
        self.position
    }
    fn cursor_direction(&self) -> Direction {
        self.direction
    }

    fn stack(&self) -> Vec<Value> {
        self.stack.clone()
    }
    fn stdout(&self) -> &str {
        &self.output
    }
    fn stdin(&mut self) -> &mut String {
        &mut self.input_buffer
    }
    fn graphics(&mut self) -> Option<&mut Graphics> {
        self.graphics.as_mut()
    }

    fn pos_history(&mut self) -> &mut HashMap<Position, Visited> {
        &mut self.pos_history
    }
    fn get_history(&mut self) -> &mut HashMap<Position, Instant> {
        &mut self.get_history
    }
    fn put_history(&mut self) -> &mut HashMap<Position, Instant> {
        &mut self.put_history
    }
    fn breakpoints(&mut self) -> &mut HashSet<Position> {
        &mut self.breakpoints
    }

    fn serialize(&self) -> String {
        self.map.serialize()
    }
}
