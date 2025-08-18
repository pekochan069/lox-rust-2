#[derive(Debug)]
#[repr(usize)]
pub enum OpCode {
    Return,
    Constant,
    Unknown,
}

impl OpCode {
    pub fn from_usize(num: usize) -> Self {
        match num {
            0 => Self::Return,
            1 => Self::Constant,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

impl Loc {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub instructions: Vec<usize>,
    pub constants: Vec<f64>,
    pub loc: Vec<Loc>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            constants: vec![],
            loc: vec![],
        }
    }

    pub fn write(&mut self, op: usize, line: Loc) {
        self.instructions.push(op);
        self.loc.push(line);
    }

    pub fn add_constant(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
        self.constants.clear();
        self.loc.clear();
    }
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    pub chunk: Chunk,
    cursor: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            cursor: 0,
        }
    }

    pub fn free(&mut self) {
        self.chunk.clear();
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.cursor = 0;
        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        InterpretResult::Ok
    }
}
