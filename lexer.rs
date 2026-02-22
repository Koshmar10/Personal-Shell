use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Iter;

#[derive(Debug, Clone)]
pub enum TokenType {
    Number,
    ID,
}
#[derive(Debug, Clone)]
pub enum GrammerPart {
    Base(TokenType),
    Terminal(String),
    NonTerminal(String),
    Choice(Vec<Vec<GrammerPart>>),
    Repeat(Vec<GrammerPart>),
    Optiona(Vec<GrammerPart>),
    Group(Vec<GrammerPart>),
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub parts: Vec<GrammerPart>,
}

#[derive(Debug, Clone)]
pub struct Grammer {
    pub start_rule: String,
    pub rules: HashMap<String, Rule>,
    pub terminals: HashSet<String>,
    pub ignore: HashSet<String>,
}
impl Default for Grammer {
    fn default() -> Self {
        Self {
            start_rule: String::new(),
            rules: HashMap::new(),
            terminals: HashSet::new(),
            ignore: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Symbol(String),
    Number(String),
    ID(String),
}
impl Token {
    pub fn get_value(&self) -> String {
        match self {
            Token::Symbol(val) => val.clone(),
            Token::Number(val) => val.clone(),
            Token::ID(val) => val.clone(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct AstNode {
    pub rule_name: String,
    pub value: Option<String>,
    pub children: Vec<AstNode>,
}

pub fn consume_until(chars: &mut impl Iterator<Item = char>, until: char) -> String {
    let mut accumulator = String::new();
    while let Some(c) = chars.next() {
        if c != until {
            accumulator.push(c);
        } else {
            break;
        }
    }
    return accumulator;
}

pub fn parse_rule(chars: &mut impl Iterator<Item = char>) -> Vec<GrammerPart> {
    let mut current_seq = Vec::new();
    let mut all_seq: Vec<Vec<GrammerPart>> = Vec::new();
    while let Some(item) = chars.next() {
        match item {
            '}' | ')' => break,
            '<' => {
                let result = consume_until(chars, '>');
                current_seq.push(GrammerPart::NonTerminal(result));
            }
            '"' => {
                let result = consume_until(chars, '"');
                current_seq.push(GrammerPart::Terminal(result));
            }
            '|' => {
                chars.next();
                all_seq.push(current_seq);
                current_seq = Vec::new();
            }
            '{' => {
                let result = parse_rule(chars);
                current_seq.push(GrammerPart::Repeat(result));
            }
            '(' => {
                let result = parse_rule(chars);
                current_seq.push(GrammerPart::Group(result));
            }
            _ => continue,
        }
    }
    all_seq.push(current_seq.clone());
    if all_seq.len() > 1 {
        return vec![GrammerPart::Choice(all_seq.clone())];
    } else {
        return current_seq;
    }
}
pub fn get_terminals(grammer: &mut String, terminals: &mut HashSet<String>) {
    let mut grammer_chars = grammer.chars();

    while let Some(c) = grammer_chars.next() {
        match c {
            '"' => {
                let result = consume_until(&mut grammer_chars, '"');
                terminals.insert(result);
            }
            _ => continue,
        }
    }
}
pub fn build_grammer(grammer_str: &mut String) -> Option<Grammer> {
    let rules = grammer_str.split(";").collect::<Vec<&str>>();
    let mut grammer = Grammer::default();
    for rule in &rules {
        let mut trimmed_rule = rule.trim();
        if trimmed_rule.is_empty() {
            continue;
        }
        if let Some((mut name_str, mut rule_str)) = trimmed_rule.split_once("::=") {
            let rule_name = name_str.trim().replace(['<', '>'], "");
            if grammer.start_rule.is_empty() {
                grammer.start_rule = rule_name.clone();
            }
            let mut rule_str = rule_str.trim();
            let grammer_rule = Rule {
                name: rule_name.clone(),
                parts: {
                    let trimmed_rule = rule_str.trim();

                    if trimmed_rule == "STRING" {
                        vec![GrammerPart::Base(TokenType::ID)]
                    } else if trimmed_rule == "NUMBER" {
                        vec![GrammerPart::Base(TokenType::Number)]
                    } else {
                        let mut char_iter = rule_str.chars().peekable();
                        parse_rule(&mut char_iter)
                    }
                },
            };
            grammer.rules.insert(rule_name, grammer_rule);
        } else {
            panic!("Gramer invalid");
        }
    }
    get_terminals(grammer_str, &mut grammer.terminals);
    Some(grammer)
}

pub fn lex(grammar: &Grammer, expression: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut terminals: Vec<String> = grammar.terminals.iter().cloned().collect();

    terminals.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut chars = expression.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        let mut matched_terminal = false;
        let remaining_str: String = chars.clone().collect();

        for term in &terminals {
            if remaining_str.starts_with(term) {
                tokens.push(Token::Symbol(term.clone()));
                for _ in 0..term.len() {
                    chars.next();
                }
                matched_terminal = true;
                break;
            }
        }

        if matched_terminal {
            continue;
        }

        if ch.is_ascii_digit() {
            let mut number = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    number.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(Token::Number(number));
            continue;
        }

        if ch.is_alphabetic() {
            let mut id = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    id.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(Token::ID(id));
            continue;
        }

        chars.next();
    }
    tokens
}

pub fn parse(
    grammar: &Grammer,
    tokens: &Vec<Token>,
    rule_name: String,
    cursor: usize,
) -> Option<AstNode> {
    let current_rule: &Rule = grammar.rules.get(&rule_name).expect("Rule not defined");
    println!("{:?}", current_rule);
    let mut children = Vec::new();

    for part in &current_rule.parts {
        match part {
            GrammerPart::Base(val) => {
                let current_token = tokens.get(cursor).expect("Cursor out of range");
                if val == TokenType::Number && *current_token == Token::Number(_) {
                    children.push(AstNode {
                        rule_name: rule_name.to_string(),
                        value: Some(current_token.get_value()),
                        children: vec![],
                    });
                } else if val == TokenType::ID && *current_token == Token::ID(_) {
                    children.push(AstNode {
                        rule_name: rule_name.to_string(),
                        value: Some(current_token.get_value()),
                        children: vec![],
                    });
                } else {
                    panic!("Invalid expression");
                }
            }
            GrammerPart::NonTerminal(term) => {}
            GrammerPart::Terminal(term) => {
                let current_token = tokens.get(cursor).expect("Cursor out of range");
                if current_token.get_value() == *term {
                    children.push(AstNode {
                        rule_name: rule_name.to_string(),
                        value: Some(current_token.get_value()),
                        children: vec![],
                    });
                } else {
                    panic!("Invalid expression");
                }
            }
            GrammerPart::Choice(choices) => {}
            GrammerPart::Group(parts) => {}
            GrammerPart::Optiona(parts) => {}
            GrammerPart::Repeat(parts) => {}
        }
    }
    None
}
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mut grammer_file = args.get(1);
    let mut expression = args.get(2);
    match grammer_file {
        Some(file) => {
            if let Ok(mut file_bytes) = fs::read_to_string(file) {
                let grammer = build_grammer(&mut file_bytes);
                dbg!("{:?}", &grammer);
                if expression.is_none() {
                    panic!("Expression must be provided to the lexer");
                }
                if grammer.is_none() {
                    panic!("Grammer failed parsing");
                }
                let actual_grammar = grammer.unwrap();
                let tokens = lex(&actual_grammar, expression.unwrap().clone());
                println!("{:?}", tokens);
                let ast = parse(
                    &actual_grammar,
                    &tokens,
                    actual_grammar.start_rule.clone(),
                    0,
                );
                println!("{:?}", ast);
            } else {
                panic!("Unable to read file")
            };
        }
        None => panic!("Expected grammer file"),
    }
    return;
}
