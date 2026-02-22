use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Iter;
use std::sync::RwLock;

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
    Optional(Vec<GrammerPart>),
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
    END,
}
impl Token {
    pub fn get_value(&self) -> String {
        match self {
            Token::Symbol(val) => val.clone(),
            Token::Number(val) => val.clone(),
            Token::ID(val) => val.clone(),
            Token::END => "END".to_string(),
        }
    }
    pub fn matches(&self, grammer_part: &GrammerPart) -> bool {
        match (self, grammer_part) {
            (Token::Symbol(s), GrammerPart::Terminal(t)) => s == t,
            (Token::Number(n), GrammerPart::Base(TokenType::Number)) => true,
            (Token::ID(s), GrammerPart::Base(TokenType::ID)) => true,

            _ => false,
        }
    }
    pub fn is_end(&self) -> bool {
        match self {
            Token::END => true,
            _ => false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct CstNode {
    pub value: Option<String>,
    pub children: Vec<CstNode>,
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
            '}' | ')' | ']' => break,
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
            '[' => {
                let result = parse_rule(chars);
                current_seq.push(GrammerPart::Optional(result));
            }
            c if c.is_alphabetic() => {
                let word = {
                    let mut new_str = String::new();
                    new_str.push(c);
                    while let Some(next_c) = chars.next() {
                        if next_c.is_alphabetic() {
                            new_str.push(next_c);
                        } else {
                            break;
                        }
                    }
                    new_str
                };
                match word.as_str() {
                    "NUMBER" => current_seq.push(GrammerPart::Base(TokenType::Number)),
                    "ID" => current_seq.push(GrammerPart::Base(TokenType::ID)),
                    _ => {}
                }
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
            let mut char_iter = rule_str.chars().peekable();
            let grammer_rule = Rule {
                name: rule_name.clone(),
                parts: parse_rule(&mut char_iter),
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
        } else {
            panic!("Unrecognized charather");
        }

        chars.next();
    }
    tokens.push(Token::END);
    tokens
}

fn parser(
    grammar: &Grammer,
    tokens: &[Token],
    parts: Vec<GrammerPart>,
    mut cursor: &mut usize,
) -> Option<CstNode> {
    if tokens.len() <= 1 || *cursor >= tokens.len() {
        return None;
    }

    let current_token = &tokens[*cursor];
    let mut children: Vec<CstNode> = Vec::new();
    dbg!(current_token);
    if !current_token.is_end() {
        for part in parts {
            match part {
                GrammerPart::Base(_) | GrammerPart::Terminal(_) => {
                    if current_token.matches(&part) {
                        children.push(CstNode {
                            value: Some(current_token.get_value()),
                            children: vec![],
                        });
                        *cursor += 1;
                    } else {
                        return None;
                    };
                }
                GrammerPart::NonTerminal(term) => {
                    let new_rule: &Rule = &grammar.rules.get(&term).expect("Missing rule for term");

                    let new_parts = new_rule.parts.clone();

                    if let Some(child) = parser(grammar, tokens, new_parts, cursor) {
                        children.push(child);
                    } else {
                        return None;
                    }
                }
                GrammerPart::Repeat(sub_parts) => {
                    if sub_parts.len() > 1 {
                        let (entry_part, rest_of_subseq) = sub_parts.split_at(1);
                        while let Some(first_child) =
                            parser(grammar, tokens, entry_part.to_vec(), &mut cursor)
                        {
                            match parser(grammar, tokens, rest_of_subseq.to_vec(), &mut cursor) {
                                Some(second_child) => {
                                    children.extend(vec![first_child, second_child]);
                                }
                                None => return None,
                            }
                        }
                    } else {
                        while let Some(first_child) =
                            parser(grammar, tokens, sub_parts.clone(), &mut cursor)
                        {
                            children.push(first_child);
                        }
                    }
                }
                GrammerPart::Choice(choices) => {
                    for choice in &choices {
                        if let Some(choice_child) =
                            parser(grammar, tokens, choice.clone(), &mut cursor)
                        {
                            children.push(choice_child);
                            break;
                        }
                    }
                }
                GrammerPart::Optional(option) => {
                    if let Some(option_child) = parser(grammar, tokens, option.clone(), &mut cursor)
                    {
                        children.push(option_child);
                    }
                }
                GrammerPart::Group(group) => {
                    if let Some(group_child) = parser(grammar, tokens, group.clone(), &mut cursor) {
                        children.push(group_child);
                    } else {
                        return None;
                    }
                }
            }
        }
        return Some(CstNode {
            value: None,
            children: children,
        });
    }
    return None;
}
fn parse_expression(grammar: &Grammer, tokens: &[Token]) -> CstNode {
    let mut cursor = 0;
    let start_rule: &Rule = &grammar
        .rules
        .get(&grammar.start_rule)
        .expect("Missing rule for term");

    let start_parts = start_rule.parts.clone();
    if let Some(cst) = parser(grammar, tokens, start_parts, &mut cursor) {
        return cst;
    } else {
        panic!("Failed to parse expression")
    }
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
                println!("{:?}", &tokens);
                let cst = parse_expression(&actual_grammar, &tokens);
                dbg!(cst);
            } else {
                panic!("Unable to read file")
            };
        }
        None => panic!("Expected grammer file"),
    }
    return;
}
