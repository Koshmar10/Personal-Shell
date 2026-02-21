use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Iter;

#[derive(Debug, Clone)]
pub enum GrammerPart {
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
            let mut rule_str = rule_str.trim().chars();
            let grammer_rule = Rule {
                name: rule_name.clone(),
                parts: parse_rule(&mut rule_str),
            };
            grammer.rules.insert(rule_name, grammer_rule);
        } else {
            panic!("Gramer invalid");
        }
    }
    get_terminals(grammer_str, &mut grammer.terminals);
    Some(grammer)
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mut grammer_file = args.get(1);
    match grammer_file {
        Some(file) => {
            if let Ok(mut file_bytes) = fs::read_to_string(file) {
                let grammer = build_grammer(&mut file_bytes);
                dbg!("{:?}", &grammer)
            } else {
                panic!("Unable to read file")
            };
        }
        None => panic!("Expected grammer file"),
    }
    return;
}
