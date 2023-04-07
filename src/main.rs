mod lexer;
use lexer::Token;
use logos::Logos;


use std::{fs::{read_dir, ReadDir, DirEntry, read_to_string}, path::{Path, PathBuf}, collections::HashMap, env};
struct Files;

#[derive(Clone, Debug)]
struct Document {
    doc_path: PathBuf,
    token_freq_map: HashMap<String, usize>,
    token_count: usize
}

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
    let mut documents = Vec::new();
    
    println!("Indexing...");
    for entry in files {
        let file = entry.unwrap();
        if let Some(doc) = process_file(&file) {
            index.insert(doc.doc_path.clone(), doc.token_freq_map.clone());
            documents.push(doc);
        }
    }

    for tf_map in index.values() {
        for token in tf_map.keys() {
            if !idf.contains_key(token) {
                idf.insert(token.clone(), calculate_idf(token.clone(), &documents));
            }
        }
    }

    let mut tfidf: HashMap<String, Vec<(&Document, f32)>> = HashMap::new();
    
    for (token, idf) in idf.iter() {
        let mut doc_ranking: Vec<(&Document, f32)> = Vec::new();
        for doc in documents.iter() {
            let tf = calculate_tf(token.clone(), doc);
            let score = if idf != &0.0 { idf * tf } else { tf }; 
            doc_ranking.push((doc, score));
        }
        doc_ranking.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        tfidf.insert(token.clone(), doc_ranking);
    }
    
    println!("Finished indexing!");
    let args: Vec<String> = env::args().collect();
    let search_token = &args[1].to_string();

    match tfidf.get(search_token) {
        Some(results) => {
            println!("Search results for '{search_token}':");
            for (doc, score) in results {
                let path = doc.doc_path.to_str().unwrap();
                println!("    Score: {score} => Path: {path}",);
            }
        },
        None => println!("Token not found in documents")
    }
}

fn process_file(file: &DirEntry) -> Option<Document> {
    let file_type = file.file_type().unwrap(); 
    
    if file_type.is_dir() {
        println!("{file:?} is a directory... Skipping.");
        return None;
    }

    // Maybe depending on the file type we can dispatch to a content parser
    let content = read_to_string(file.path()).expect("Should be able to read");
    
    let mut tf: TF = HashMap::new();
    let mut lexer = Token::lexer(&content);
    let mut count = 0;
    while let Some(_token_type) = lexer.next() {
        count += 1;
        let token = lexer.slice().to_string();
        tf.insert(token.clone(), tf.get(&token).cloned().unwrap_or(0) + 1);
        // println!("TOKEN => {_token_type:?} = {token:?}");
    }
    Some(Document {
        doc_path: file.path(),
        token_freq_map: tf,
        token_count: count
    })
}


// https://en.wikipedia.org/wiki/Tf%E2%80%93idf
fn calculate_tf(token: String, doc: &Document) -> f32 {
    if doc.token_count == 0 { return 0.0; }
    let token_freq = doc.token_freq_map.get(&token).cloned().unwrap_or(0);
    return token_freq as f32 / doc.token_count as f32
}

fn calculate_idf(token: String, docs: &Vec<Document>) -> f32 {
    let n = docs.len() as f32;
    let m = docs.iter().fold(1.0, |acc, d| {
        if d.token_freq_map.contains_key(&token) { acc + 1.0 } else { acc }
    });
    (n / m).log10()
}
