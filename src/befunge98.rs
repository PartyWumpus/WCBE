use std::cmp;

use coarsetime::{Duration, Instant};
use egui::ahash::{HashSet, HashSetExt};
use rand::Rng;

use egui::ahash::HashMap;

use crate::{
    app::{self, Settings},
    befunge::{
        self, Befunge, FungeSpaceTrait, GraphicalEvent, Graphics, Position, Value, Visited,
        WhereVisited,
    },
};

const HANDPRINT: i64 = 0x8B38669B;
const VERSION: i64 = 100;

#[derive(Debug)]
pub enum StepStatus {
    Normal,
    NormalNoStep,
    Breakpoint,
    Die,
    Error(&'static str),
    SyncFrame,
    Clone,
}

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

#[derive(Clone)]
pub struct State {
    state: StateTempName,
    cursors: Vec<Cursor>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cursors: vec![Cursor::default()],
            state: StateTempName::default(),
        }
    }
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
    pub storage_offset: Position,
    pub stacks: Vec<Vec<Value>>,
    pub position: Position,
    pub direction: Direction,
    pub string_mode: bool,
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
            storage_offset: (0, 0),
            string_mode: false,
            position: (0, 0),
            direction: Direction::East,
            stacks: vec![vec![]],
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

    #[deprecated]
    pub fn step_position(&mut self, settings: &Settings) {
        for cursor in &mut self.cursors {
            cursor.step_position(&mut self.state, settings);
        }
    }

    #[deprecated]
    pub fn reflect(&mut self) {
        for cursor in &mut self.cursors {
            cursor.direction = cursor.direction.reverse();
        }
    }

    pub fn step(&mut self, settings: &Settings) -> befunge::StepStatus {
        let mut new = Vec::new();
        let mut deleted = Vec::new();
        let mut breakpoint = false;
        for (i, cursor) in (self.cursors).iter_mut().enumerate() {
            match cursor.step(&mut self.state, settings) {
                StepStatus::Clone => {
                    let mut copy = cursor.clone();
                    copy.direction = copy.direction.reverse();
                    copy.step_position(&mut self.state, settings);
                    new.push(copy);
                    cursor.step_position(&mut self.state, settings);
                }
                StepStatus::Die => {
                    deleted.push(i);
                }
                StepStatus::Error(str) => {
                    use app::InvalidOperationBehaviour as IOpBehav;
                    match settings.invalid_operation_behaviour {
                        IOpBehav::Reflect => {
                            cursor.direction = cursor.direction.reverse();
                            cursor.step_position(&mut self.state, settings);
                        }
                        IOpBehav::Halt => return befunge::StepStatus::Error(str),
                        IOpBehav::Ignore => (),
                    }
                }
                StepStatus::Breakpoint => {
                    breakpoint = true;
                }
                _ => (),
            };
        }
        self.cursors.append(&mut new);

        // slow af :pensive:
        for i in deleted.into_iter().rev() {
            if self.cursors.len() == 1 {
                return befunge::StepStatus::Breakpoint;
            } else {
                self.cursors.remove(i);
            }
        }

        if breakpoint {
            befunge::StepStatus::Breakpoint
        } else {
            befunge::StepStatus::Normal
        }
    }
}

impl Cursor {
    fn toss(&mut self) -> &mut Vec<Value> {
        self.stacks.last_mut().unwrap()
    }

    fn soss(&mut self) -> Option<&mut Vec<Value>> {
        let mut rev = self.stacks.iter_mut().rev();
        rev.next().unwrap();
        rev.next()
    }

    fn toss_and_soss(&mut self) -> (&mut Vec<Value>, Option<&mut Vec<Value>>) {
        let mut rev = self.stacks.iter_mut().rev();
        (rev.next().unwrap(), rev.next())
    }

    fn transfer_n_to_toss(&mut self, n: Value) {
        if n < 0 {
            for _ in n..0 {
                self.soss().unwrap().push(0)
            }
            return;
        }

        if let (toss, Some(soss)) = self.toss_and_soss() {
            let len = soss.len() as i64;
            if len <= n {
                for _ in 0..(len - n) {
                    toss.push(0)
                }
                toss.append(soss);
            } else {
                let mut arr = soss.split_off((len - n) as usize);
                toss.append(&mut arr);
            }
        } else {
            for _ in 0..n {
                self.toss().push(0)
            }
        }
    }

    fn transfer_n_to_soss(&mut self, n: Value) {
        if n < 0 {
            for _ in n..0 {
                self.soss().unwrap().pop();
            }
            return;
        }

        if let (toss, Some(soss)) = self.toss_and_soss() {
            let len = toss.len() as i64;
            if len <= n {
                for _ in 0..(len - n) {
                    soss.push(0)
                }
                soss.append(toss);
            } else {
                let mut arr = toss.split_off((len - n) as usize);
                soss.append(&mut arr);
            }
        } else {
            for _ in 0..n {
                self.soss().unwrap().push(0)
            }
        }
    }

    fn pop(&mut self) -> Value {
        self.toss().pop().unwrap_or(0)
    }

    fn push(&mut self, val: Value) {
        self.toss().push(val);
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
                self.push(op);
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

            b'0'..=b'9' => self.push((op - b'0').into()),

            // 2 op operations
            b'+' => {
                let a = self.pop();
                let b = self.pop();
                self.push(b + a);
            }
            b'-' => {
                let a = self.pop();
                let b = self.pop();
                self.push(b - a);
            }
            b'*' => {
                let a = self.pop();
                let b = self.pop();
                self.push(b * a);
            }
            b'/' => {
                let a = self.pop();
                let b = self.pop();
                if a == 0 {
                    self.push(0);
                } else {
                    self.push(b / a);
                }
            }
            b'%' => {
                let a = self.pop();
                let b = self.pop();
                if a == 0 {
                    self.push(0);
                } else {
                    self.push(b % a);
                }
            }
            b'`' => {
                let a = self.pop();
                let b = self.pop();
                self.push(if b > a { 1 } else { 0 });
            }
            b'\\' => {
                let a = self.pop();
                let b = self.pop();
                self.push(a);
                self.push(b);
            }

            // one op operations
            b'!' => {
                let a = self.pop();
                self.push(if a == 0 { 1 } else { 0 });
            }
            b':' => {
                let a = self.pop();
                self.push(a);
                self.push(a);
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
            b'?' => {
                let x: u8 = rand::thread_rng().r#gen_range(0..4);
                self.direction = match x {
                    0 => Direction::North,
                    1 => Direction::South,
                    2 => Direction::East,
                    3 => Direction::West,
                    _ => unreachable!(),
                };
            }
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

            // put
            b'p' => {
                let y = self.pop() + self.storage_offset.1;
                let x = self.pop() + self.storage_offset.0;
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
                let y = self.pop() + self.storage_offset.1;
                let x = self.pop() + self.storage_offset.0;
                self.push(state.map.get((x, y)));

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
                                self.push(-1);
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
                            self.push(num);
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
                            self.push(-1);
                        } else {
                            return StepStatus::Breakpoint;
                        }
                    }
                    Some(chr) => {
                        self.push(chr as Value);
                        state.input_buffer = itr.as_str().into();
                    }
                }
            }

            // halt is dealt with higher up
            b'@' => return StepStatus::Die,

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
                self.push(state.map.get(self.position));
                self.step_position_inner(state);
                return StepStatus::NormalNoStep;
            }

            b';' => loop {
                self.step_position_inner(state);
                let op = state.map.get(self.position);
                if op == b';' as Value {
                    break;
                }
            },

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

            b'a'..=b'f' => self.push((op - b'a' + 10).into()),

            b'j' => {
                let count = self.pop();

                if count < 0 {
                    self.direction = self.direction.reverse();
                    for _ in count..-2 {
                        self.step_position_inner(state);
                    }
                    self.direction = self.direction.reverse();
                    return StepStatus::NormalNoStep;
                } else {
                    for _ in 0..count + 1 {
                        self.step_position_inner(state);
                    }
                }
                return StepStatus::NormalNoStep;
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
                let (a, b) = self.position;
                self.position = (x, y);

                if let Ok(op) = op.try_into() {
                    for _ in 0..count + 1 {
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

                self.position = (a, b);
                return StepStatus::Normal;
            }

            b'n' => {
                self.toss().clear();
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

            b't' => return StepStatus::Clone,

            b'u' => {
                let count = self.pop();

                if let (toss, Some(soss)) = self.toss_and_soss() {
                    if count < 0 {
                        for _ in count..0 {
                            soss.push(toss.pop().unwrap_or(0));
                        }
                    } else {
                        for _ in 0..count {
                            toss.push(soss.pop().unwrap_or(0));
                        }
                    }
                } else {
                    return StepStatus::Error("no soss?");
                }
            }

            b'w' => {
                let a = self.pop();
                let b = self.pop();
                match a.cmp(&b) {
                    cmp::Ordering::Greater => self.direction = self.direction.turn_left(),

                    cmp::Ordering::Less => self.direction = self.direction.turn_right(),
                    cmp::Ordering::Equal => (),
                }
            }

            b'y' => {
                let count = self.pop();
                let original_size = self.toss().len();

                let mut flags = 0;
                flags |= 0b1; // t (conncurrent funge)
                //flags |= 0b10; // i (file input)
                //flags |= 0b100; // o (file output)
                //flags |= 0b1000; // = (exec)
                //flags |= 0b10000; // unbuffered IO

                let stack_sizes: Vec<Value> =
                    self.stacks.iter().map(|s| s.len() as Value).rev().collect();

                self.push(0); // env vars
                self.push(0); // cli args

                for stack_size in stack_sizes {
                    self.push(stack_size); // size of stack stack
                }
                self.push(self.stacks.len() as Value); // size of stack stack

                // (hour * 256 * 256) + (minute * 256) + (second)
                self.push(0);

                // ((year - 1900) * 256 * 256) + (month * 256) + (day of month)
                self.push(0);

                // Bottom right + top left (TODO:)
                self.push(state.map.max_size.0);
                self.push(state.map.max_size.1);

                // Top left (TODO:)
                self.push(0);
                self.push(0);

                // Storage delta
                self.push(self.storage_offset.0);
                self.push(self.storage_offset.1);

                // IP delta
                self.push(self.direction.0);
                self.push(self.direction.1);

                // IP position
                self.push(self.position.0);
                self.push(self.position.1);

                self.push(0); // Team ID???
                self.push(0); // IP ID //FIXME:
                self.push(2); // num dimensions
                self.push(std::path::MAIN_SEPARATOR as Value); // path seperator
                self.push(0); // exec behaviour
                self.push(VERSION); // version
                self.push(HANDPRINT); // handprint
                self.push(8); // bytes per cell
                self.push(flags); // flags

                if count > 0 {
                    let len = self.toss().len();
                    let ret = *self.toss().get(len - count as usize).unwrap_or(&0);
                    self.toss().truncate(original_size);
                    self.toss().push(ret);
                }
            }

            b'{' => {
                let count = self.pop();
                self.stacks.push(vec![]);
                self.transfer_n_to_toss(count);
                let storage_offset = self.storage_offset;
                self.soss().unwrap().push(storage_offset.1);
                self.soss().unwrap().push(storage_offset.0);

                // TODO: add a "peek position" function
                let (x, y) = self.position;
                self.step_position_inner(state);
                self.storage_offset = self.position;
                self.position = (x, y);
            }
            b'}' => {
                let count = self.pop();
                if let Some(soss) = self.soss() {
                    self.storage_offset = (soss.pop().unwrap_or(0), soss.pop().unwrap_or(0));

                    self.transfer_n_to_soss(count);

                    self.stacks.pop();
                } else {
                    return StepStatus::Error("no soss?");
                }
            }

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
    fn step(&mut self, settings: &Settings) -> befunge::StepStatus {
        self.step(settings)
    }

    fn program_size(&self) -> Position {
        self.state.map.max_size
    }
    fn instruction_count(&self) -> usize {
        self.state.instruction_count
    }
    fn string_mode(&self) -> bool {
        self.cursors[0].string_mode
    }
    fn cursor_positions(&self) -> Vec<Position> {
        self.cursors.iter().map(|c| c.position).collect()
    }
    fn cursor_direction(&self) -> (Value, Value) {
        (self.cursors[0].direction.0, self.cursors[0].direction.1)
    }

    fn stack(&self) -> Vec<Value> {
        // TODO:
        self.cursors[0].stacks.last().unwrap().clone()
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
