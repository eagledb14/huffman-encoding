use std::fs;
use std::fs::File;
use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::io::Read;
use std::io::Write;
use std::env;


fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() < 4 {
        panic!("Incorrect thing");
    }

    let compute_type = args[1].as_str();
    let mut filename = "".to_owned();
    let mut savename = "temp".to_owned();
    match compute_type {
        "-c" => {
            filename = args[2].clone();
            savename = args[3].clone();
        }
        "-d" => {
            filename = args[2].clone();
            savename = args[3].clone();
        }
        _ => {
            panic!("Unknown flag, please only use -c and -d ")
        }
    }


    match args[1].as_str() {
        "-c" => {
            let input = fs::read_to_string(filename).expect("file read");
            let sorted_chars = parse_file(input.clone());
            let tree = create_huffman(sorted_chars);
            let mut encoding = HashMap::<String, String>::new();
            get_encoding(tree.clone(), &mut encoding, "".to_string());

            compress(input, encoding, savename);
        }
        "-d" => {
            decompress(filename, savename);
        }
        _ => {
            panic!("Unknown flag, please only use -c and -d ")
        }
    }
}

fn compress(text: String, encoding: HashMap<String, String>, filename: String) {
    let mut encoded = "".to_owned();
    let header = create_header(encoding.clone());

    for c in text.chars() {
        encoded += if let Some(x) = encoding.get(&c.to_string()) {
            x
        } else {
            continue;
        }
    }

    let bits: Vec<u8> = encoded.chars()
        .map(|c| if c == '1' { 1 } else { 0 })
        .collect();

    let mut file = File::create(filename).expect("");

    let _ = file.write_all(header.as_bytes());
    let _ = file.write_all(&bits);
}

fn decompress(filename: String, savename: String) {
    let text = fs::read_to_string(filename.clone()).expect("file read");
    let (encoding, length) = read_header(text.clone());

    // this is done to remove the header, so that the rest of the file can be read 
    // in a different buffer
    let mut header_buffer = vec![0 as u8; length];
    let mut file = File::open(filename).expect("");
    file.read_exact(&mut header_buffer).expect("");

    let mut body = Vec::<u8>::new();
    file.read_to_end(&mut body).expect("");

    let bit_string: String = body.into_iter().map(|b| if b == 1 { '1' } else { '0' }).collect();
    // println!("{}", bit_string);

    let mut decompressed_file = Vec::<String>::new();
    let mut code = "".to_owned();

    println!("{:?}", encoding);
    println!("{}", bit_string);

    for c in bit_string.chars().map(|x| x.to_string()) {
        if let Some(value) = encoding.get(&code) {
            decompressed_file.push(value.to_owned());
            code = c;
        } else {
            code += &c;
        }
    }
    println!("{}", decompressed_file.join(""));

    let mut file = File::create(savename).expect("");
    let _ = file.write_all(decompressed_file.join("").as_bytes());
}


fn create_header(encoding: HashMap<String, String>) -> String {
    let mut parts = Vec::<String>::new();

    for (key, value) in encoding.iter() {
        parts.push(format!(":{}{}", key, value));
    }
    let head = parts.join("");

    return head.len().to_string() + &head;
}

fn read_header(mut text: String) -> (HashMap<String, String>, usize) {
    let mut encoding = HashMap::<String, String>::new();

    let token_num = if let Some(index) = text.find(":") {
        match text[0..index].to_string().parse::<usize>() {
            Ok(len) => {
                text = text[index + 1..].to_string();
                len
            }
            Err(_) => panic!("Error: Malformed text")
        }
    } else {
        panic!("Error: Malformed text");
    };

    let mut txt_iter = text.chars();
    let mut symbol = txt_iter.next().unwrap().to_string();
    let mut code = "".to_owned();

    for _ in 0..token_num {
        let c = if let Some(x) = txt_iter.next() {
            x.to_string()
        } else {
            txt_iter.next();
            continue;
        };

        if c == ":" {
            encoding.insert(code, symbol);

            symbol = txt_iter.next().unwrap().to_string();
            code = "".to_owned();
        } else {
            code += &c.to_string();
        }
    }

    return (encoding, token_num + token_num.to_string().len());
}

fn get_encoding(node: Box<Node>, codes: &mut HashMap<String, String>, code: String) {

    if let Some(name) = node.name {
        codes.insert(name, code);
    } else {
        if let Some(left) = node.left {
            get_encoding(left, codes, code.clone() + "0");
        }
        if let Some(right) = node.right {
            get_encoding(right, codes, code.clone() + "1");
        }
    }
}

fn parse_file(input: String) -> HashMap<String, u32> {
    let mut chars = HashMap::<String, u32>::new();

    for char in input.chars().into_iter() {
        *chars.entry(char.to_string()).or_insert(0) += 1;
    }

    return chars;
}

fn create_huffman(sorted_chars: HashMap<String, u32>) -> Box<Node> {
    let mut tree = BinaryHeap::<Reverse<Node>>::new();
    for (name, weight) in sorted_chars.iter() {
        tree.push(Reverse(Node::new(name.clone(), *weight)));
    }

    while tree.len() > 1 {
        let left = tree.pop().unwrap().0;
        let right = tree.pop().unwrap().0;

        let mut body = Node::new_body(left.frequency + right.frequency);
        body.left = Some(Box::new(left));
        body.right = Some(Box::new(right));
        tree.push(Reverse(body));
    }

    return Box::new(tree.pop().unwrap().0);
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Node {
    name: Option<String>,
    pub frequency: u32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    pub fn new(name: String, frequency: u32) -> Self {
        return Self {
            name: Some(name),
            frequency,
            left: None,
            right: None
        }
    }

    pub fn new_body(frequency: u32) -> Self {
        return Self {
            name: None,
            frequency,
            left: None,
            right: None,
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.frequency.cmp(&other.frequency);
    }
}

