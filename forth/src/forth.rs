use std::collections::VecDeque;

type WordExecutor = Fn(&Word, &mut Vec<Value>, &mut VecDeque<Token>) -> ForthResult;

struct Word {
    name: String,
    data: Vec<Token>,
    exec: &'static WordExecutor,
}

impl Word {
    fn new(name: &str, exec: &'static WordExecutor) -> Self {
        Self {
            name: String::from(name),
            data: Vec::new(),
            exec,
        }
    }

    fn new_compiled(name: &str, tokens: Vec<Token>) -> Self {
        Self {
            name: String::from(name),
            data: tokens,
            exec: &do_exec,
        }
    }
}

fn do_nop(_word: &Word, _stack: &mut Vec<Value>, _tokens: &mut VecDeque<Token>) -> ForthResult {
    Ok(())
}

fn do_arithmetic(word: &Word, stack: &mut Vec<Value>, _tokens: &mut VecDeque<Token>) -> ForthResult {
    if stack.len() < 2 {
        return Err(Error::StackUnderflow);
    }
    let v2 = stack.pop().unwrap();
    let v1 = stack.pop().unwrap();
    let v = match word.name.as_str() {
        "+" => v1 + v2,
        "-" => v1 - v2,
        "*" => v1 * v2,
        "/" => {
            if v2 == 0 {
                return Err(Error::DivisionByZero);
            }
            v1 / v2
        },
        _ => unreachable!(),
    };
    stack.push(v);
    Ok(())
}

fn do_dup(_word: &Word, stack: &mut Vec<Value>, _tokens: &mut VecDeque<Token>) -> ForthResult {
    if stack.len() < 1 {
        return Err(Error::StackUnderflow);
    }
    let v = *stack.last().unwrap();
    stack.push(v);
    Ok(())
}

fn do_drop(_word: &Word, stack: &mut Vec<Value>, _tokens: &mut VecDeque<Token>) -> ForthResult {
    if stack.len() < 1 {
        return Err(Error::StackUnderflow);
    }
    stack.pop().unwrap();
    Ok(())
}

fn do_swap(_word: &Word, stack: &mut Vec<Value>, _tokens: &mut VecDeque<Token>) -> ForthResult {
    if stack.len() < 2 {
        return Err(Error::StackUnderflow);
    }
    let v1 = stack.pop().unwrap();
    let v2 = stack.pop().unwrap();
    stack.push(v1);
    stack.push(v2);
    Ok(())
}

fn do_over(_word: &Word, stack: &mut Vec<Value>, _tokens: &mut VecDeque<Token>) -> ForthResult {
    if stack.len() < 2 {
        return Err(Error::StackUnderflow);
    }
    let v = stack[stack.len() - 2];
    stack.push(v);
    Ok(())
}

fn do_exec(word: &Word, _stack: &mut Vec<Value>, tokens: &mut VecDeque<Token>) -> ForthResult {
    for token in word.data.iter().rev() {
        tokens.push_front(token.clone());
    }
    Ok(())
}

pub type Value = i32;
pub type ForthResult = Result<(), Error>;

pub struct Forth {
    stack: Vec<Value>,
    tokens: VecDeque<Token>,
    words: Vec<Word>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

#[derive(Debug, Clone)]
enum Token {
    Word(String),
    WordIndex(usize),
    Number(Value),
}

fn parse(s: &str) -> VecDeque<Token> {
    let items: Vec<String> = s.split(|c: char| c.is_whitespace() || c.is_ascii_control())
                              .filter(|k| !k.is_empty()).map(String::from).collect();

    let mut tokens = VecDeque::new();
    for s in items {
        let t = match s.parse::<Value>() {
            Ok(v) => Token::Number(v),
            Err(_) => Token::Word(s.to_uppercase()),
        };
        tokens.push_back(t);
    }
    tokens
}

impl Forth {
    pub fn new() -> Forth {
        let mut words = Vec::new();
        for name in ["+", "-", "*", "/"].iter() {
            words.push(Word::new(name, &do_arithmetic));
        }
        words.push(Word::new("DUP", &do_dup));
        words.push(Word::new("DROP", &do_drop));
        words.push(Word::new("SWAP", &do_swap));
        words.push(Word::new("OVER", &do_over));
        words.push(Word::new(":", &do_nop));
        Self {
            stack: Vec::new(),
            tokens: VecDeque::new(),
            words,
        }
    }

    pub fn stack(&self) -> Vec<Value> {
        self.stack.clone()
    }

    fn lookup_word(&self, name: &str) -> Option<usize> {
        for (i, w) in self.words.iter().rev().enumerate() {
            let i = self.words.len() - 1 - i;
            if w.name == name {
                return Some(i)
            }
        }
        None
    }

    fn compile(&mut self) -> ForthResult {
        let word_name = if let Some(Token::Word(word)) = self.tokens.pop_front() {
            word
        } else {
            return Err(Error::InvalidWord);
        };

        let mut word_tokens = Vec::new();
        while let Some(token) = self.tokens.pop_front() {
            match token {
                Token::Word(ref name) if name == ";" => {
                    self.words.push(Word::new_compiled(&word_name, word_tokens));
                    return Ok(())
                }
                Token::Word(name) => {
                    if let Some(index) = self.lookup_word(name.as_str()) {
                        word_tokens.push(Token::WordIndex(index));
                    } else {
                        return Err(Error::InvalidWord);
                    }
                }
                _ => {
                    word_tokens.push(token);
                }
            }
        }
        Err(Error::InvalidWord)
    }

    fn interp(&mut self) -> ForthResult {
        let compile_index = self.lookup_word(":").unwrap();
        let t = self.tokens.pop_front().unwrap();
        match t {
            Token::Word(word) => {
                if let Some(word_index) = self.lookup_word(&word) {
                    self.tokens.push_front(Token::WordIndex(word_index));
                } else {
                    return Err(Error::UnknownWord);
                }
            }
            Token::WordIndex(index) if index == compile_index => {
                self.compile()?;
            }
            Token::WordIndex(index) => {
                let word = &self.words[index];
                (word.exec)(&word, &mut self.stack, &mut self.tokens)?;
            }
            Token::Number(v) => self.stack.push(v),
        }
        Ok(())
    }

    pub fn eval(&mut self, input: &str) -> ForthResult {
        self.tokens = parse(input);
        while self.tokens.len() > 0 {
            self.interp()?;
        }
        Ok(())
    }
}

