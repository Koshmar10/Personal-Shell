// 1 + 2 - 3
//   +
//  / \
// 1   \
//  \   -
//     / \
//    2   3
// Add
// ├── 1
// └── Multiply
//     ├── 2
//     └── 3
// 1 + (2 - 3)
//
use std::{collections::HashMap, env, process::exit};

const OPERANDS: [&str; 4] = ["+", "-", "/", "*"];
const DELIMITERS: [&str; 2] = ["(", ")"];

#[derive(Debug, Clone)]
pub enum OpType {
    Add(u8),
    Multiply(u8),
    Divide(u8),
    Subtract(u8),
}
impl OpType {
    pub fn get_priority(&self) -> u8 {
        match self {
            OpType::Add(a) => return *a,
            OpType::Divide(a) => return *a,
            OpType::Multiply(a) => return *a,
            OpType::Subtract(a) => return *a,
        }
    }
}
// tokenization data structuire
#[derive(Debug, Clone)]
pub enum Token {
    Op(OpType),
    Delimiter(String),
    Lit(i32),
}
impl Token {
    pub fn get_op(&self) -> OpType {
        match self {
            Token::Op(op) => op.clone(),
            _ => panic!("Expected an operator, but found something else!"),
        }
    }
}
#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: OpType,
        right: Box<Expr>,
    },
    Lit(i32),
}

fn main() {
    let op_priority = build_proirity_map();

    let args: Vec<String> = env::args().collect();

    let tokens = tokenizer(&args[1], &op_priority);

    let mode = args.get(2);
    if let Some(mode) = mode {
        let m = mode.as_str();
        let exp = parse_expression(&tokens[0..]);

        match m {
            "-p" | "--print" => {
                print_expression(&exp, "".to_string(), false, true);
            }
            "-e" | "--eval" => {
                let eval = eval_expression(&exp);
                print!("Result: {}", eval);
            }
            _ => {}
        }
    } else {
        exit(1);
    }
}

fn build_proirity_map() -> HashMap<&'static str, OpType> {
    let mut op_priority: HashMap<&str, OpType> = HashMap::new();
    op_priority.insert("+", OpType::Add(2));
    op_priority.insert("-", OpType::Subtract(2));
    op_priority.insert("/", OpType::Divide(1));
    op_priority.insert("*", OpType::Multiply(1));
    return op_priority;
}

fn tokenizer(input_expression: &str, op_priority: &HashMap<&'static str, OpType>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let chunks = input_expression
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    for chunk in &chunks {
        let chunk_str = &chunk.as_str();

        if OPERANDS.contains(chunk_str) {
            let priority: OpType = op_priority
                .get(chunk_str)
                .cloned()
                .expect("Expression containes undefined opperand");
            tokens.push(Token::Op(priority));
        } else if DELIMITERS.contains(chunk_str) {
            tokens.push(Token::Delimiter(chunk.clone()));
        } else {
            if !chunk.contains(" ") {
                tokens.push(Token::Lit(chunk.parse::<i32>().unwrap()));
            }
        }
    }

    return tokens;
}

fn parse_expression(tokens: &[Token]) -> Expr {
    if tokens.len() == 1 {
        match tokens[0] {
            Token::Lit(val) => return Expr::Lit(val),
            _ => panic!("Expected literal, found {:?}", tokens[0]),
        }
    }

    let mut paranthess_level = 0;
    let mut pivot: Option<usize> = None;
    let mut current_max: u8 = 0;

    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Delimiter(d) if d == "(" => paranthess_level += 1,
            Token::Delimiter(d) if d == ")" => paranthess_level -= 1,
            Token::Op(kind) if paranthess_level == 0 => {
                let pr = kind.get_priority();
                if pr >= current_max {
                    current_max = pr;
                    pivot = Some(i);
                }
            }
            _ => {}
        }
    }

    if let Some(idx) = pivot {
        return Expr::Binary {
            left: Box::new(parse_expression(&tokens[0..idx])),
            op: tokens[idx].get_op(),
            right: Box::new(parse_expression(&tokens[idx + 1..])),
        };
    }

    if tokens.len() >= 2 && matches!(&tokens[0], Token::Delimiter(d) if d == "(") {
        return parse_expression(&tokens[1..tokens.len() - 1]);
    }

    panic!("Parse error at tokens: {:?}", tokens);
}

fn print_expression(expr: &Expr, indent: String, is_last: bool, is_first: bool) {
    let mut marker = "│  ";
    if !is_first {
        marker = if is_last { "└─" } else { "├─" };
    }

    match expr {
        Expr::Lit(val) => {
            println!("{}{}{}", indent, marker, val);
        }
        Expr::Binary { left, op, right } => {
            let op_char = match op {
                OpType::Add(_) => "+",
                OpType::Subtract(_) => "-",
                OpType::Multiply(_) => "*",
                OpType::Divide(_) => "/",
            };
            println!("{}{}{}", indent, marker, op_char);

            let new_indent = if is_last {
                format!("{}  ", indent)
            } else {
                format!("{}│  ", indent)
            };

            print_expression(left, new_indent.clone(), false, false);
            print_expression(right, new_indent, true, false);
        }
    }
}

fn eval_expression(expr: &Expr) -> i32 {
    match expr {
        Expr::Lit(v) => return *v,
        Expr::Binary { left, op, right } => match op {
            OpType::Add(_) => {
                return eval_expression(left) + eval_expression(right);
            }
            OpType::Divide(_) => {
                return eval_expression(left) / eval_expression(right);
            }
            OpType::Multiply(_) => {
                return eval_expression(left) * eval_expression(right);
            }
            OpType::Subtract(_) => {
                return eval_expression(left) - eval_expression(right);
            }
        },
    }
}
