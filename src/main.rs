mod lexer;
use lexer::Token;
use logos::Logos;


use std::{fs::{read_dir, ReadDir, DirEntry, read_to_string}, path::{Path, PathBuf}, collections::HashMap};
struct Files;

impl FileProvider for Files {
    fn files(dir: &Path) -> ReadDir {
       read_dir(dir).expect("Could not read dir")
    }
}

trait FileProvider {
    fn files(dir: &Path) -> ReadDir;
}

type TF = HashMap<String, usize>;
type IDF = HashMap<String, f32>;
type Index = HashMap<PathBuf, TF>;

fn main() {
    let dir = Path::new("./sample_content/");
    let files = Files::files(&dir);
    
    let mut index: Index = HashMap::new();
    let mut idf: IDF = HashMap::new();
    
    println!("Indexing...");
    let mut documents = Vec::new();
    for entry in files {
        let file = entry.unwrap();
        if let Some(doc) = process_file(&file) {
            index.insert(file.path(), doc.token_freq_map);
            documents.push(doc);
        }
    }

    for tf_map in index.values() {
        for &token in tf_map.keys() {
            let f: f32 = idf.get(&token).unwrap_or(&0.0) + 1.0;
            idf.insert(token, f);
            idf.insert(token, calculate_idf(token, &documents));
        }
    }

    let mut tfidf: HashMap<String, Vec<(&PathBuf, f32)>> = HashMap::new();

    for (token, idf) in idf.iter() {
        for (path, tf_map) in index.iter() {
            if tf_map.contains_key(token) {
                let score = idf * tf_map.get(token).unwrap();

                let mut ranking = tfidf.clone().get(token).unwrap_or(&Vec::new()).to_owned();
                ranking.push((path, score));
                ranking.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
                tfidf.insert(token.clone(), ranking);
            } 
        }
    }

    //dbg!(&index);
    //dbg!(idf);
    //dbg!(tfidf);
    println!("Search for 'James': ");
    // dbg!(tfidf.get("James"));
    
}

fn process_file(file: &DirEntry) -> Option<Document> {
    let file_type = file.file_type().unwrap(); 
    
    if file_type.is_dir() {
        println!("{file:?} is a directory... Skipping.");
        return None;
    }

    // Maybe depending on the file type we can dispatch to a content parser
    let content = read_to_string(file.path()).expect("Should be able to read");
    println!("==================");
    
    let mut tf: TF = HashMap::new();
    let mut doc = Document {
        token_freq_map: tf,
        token_count: 0
    };
    let mut lexer = Token::lexer(&content);
    while let Some(token_type) = lexer.next() {
        doc.token_count += 1;
        let token = lexer.slice().to_string();
        tf.insert(token, tf.get(&token).unwrap_or(&0) + 1);
        println!("TOKEN => {token_type:?} = {token:?}");
    }
    return Some(doc);
}

struct Document {
    token_freq_map: HashMap<String, usize>,
    token_count: usize
}

// https://en.wikipedia.org/wiki/Tf%E2%80%93idf
fn calculate_tf(token: String, doc: &Document) -> f32 {
    let token_freq = doc.token_freq_map.get(&token).cloned().unwrap_or(0);
    let total_tokens = if doc.token_count > 0 {doc.token_count} else {1};
    (token_freq / total_tokens) as f32
}

fn calculate_idf(token: String, docs: &Vec<Document>) -> f32 {
    let n = docs.len() as f32;
    let m = docs.iter().fold(1.0, |acc, d| {
        if d.token_freq_map.contains_key(&token) { acc + 1.0 } else { acc }
    });
    (n / m).log10()
}

