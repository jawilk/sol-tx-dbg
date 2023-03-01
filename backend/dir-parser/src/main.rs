use serde::{Deserialize, Serialize};
use serde_json;
use walkdir::WalkDir;


#[derive(Debug, Serialize)]
struct Path {
    full: String,
    parts: Vec<String>,
}
impl Path {
    pub fn new(full: &str, path: &str) -> Path {
        Path {
            full: full.to_string(),
            parts: path.to_string().split("/").map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Dir {
    name: String,
    children: Vec<Box<Dir>>,
    path: String,
}

impl Dir {
    fn new(name: &str, path: &str) -> Dir {
        Dir {
            name: name.to_string(),
            children: Vec::<Box<Dir>>::new(),
            path: path.to_string(),
        }
    }

    fn find_child(&mut self, name: &str) -> Option<&mut Dir> {
        for c in self.children.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<Dir>,
    {
        self.children.push(Box::new(leaf.into()));
        self
    }
}

fn dir(val: &str) -> Dir {
    Dir::new(val, "")
}

fn main() {
    let mut paths = vec![];

    for entry in WalkDir::new("code") {
        let tmp = entry.unwrap();
        paths.push(Path::new(
            tmp.path().to_str().unwrap(),
            tmp.path().strip_prefix("code/").unwrap().to_str().unwrap(),
        ));
    }

    let mut top = dir("code");
    for path in paths.iter() {
        build_tree(&mut top, &path.parts, 0, &path.full);
    }

    println!(
        "Intermediate Representation of Dirs:\n{:#?}\n\nOutput Tree Format:\n",
        top
    );

    print_dir(&top, 0);

    let res_string = serde_json::to_string(&top).unwrap();
    println!("{}", res_string);
    std::fs::write("test.json", res_string).unwrap();
}

fn build_tree(node: &mut Dir, parts: &Vec<String>, depth: usize, path_full: &str) {
    if depth < parts.len() {
        let item = &parts[depth];

        let mut dir = match node.find_child(&item) {
            Some(d) => d,
            None => {
                let d = Dir::new(&item, path_full);
                node.add_child(d);
                match node.find_child(&item) {
                    Some(d2) => d2,
                    None => panic!("Got here!"),
                }
            }
        };
        build_tree(&mut dir, parts, depth + 1, path_full);
    }
}

// A function to print a Dir in format similar to unix `tree` command.
fn print_dir(dir: &Dir, depth: u32) {
    if depth == 0 {
        println!("{}", dir.name);
    } else {
        println!(
            "{:indent$}{} {}",
            "",
            "└──",
            dir.name,
            indent = ((depth as usize) - 1) * 4
        );
    }

    for child in dir.children.iter() {
        print_dir(child, depth + 1)
    }
}
