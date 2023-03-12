use serde::Serialize;
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
            parts: path.to_string().split('/').map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Dir {
    name: String,
    children: Vec<Dir>,
    path: String,
}

impl Dir {
    fn new(name: &str, path: &str) -> Dir {
        Dir {
            name: name.to_string(),
            children: Vec::<Dir>::new(),
            path: path.to_string(),
        }
    }

    fn find_child(&mut self, name: &str) -> Option<&mut Dir> {
        self.children.iter_mut().find(|c| c.name == name)
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<Dir>,
    {
        self.children.push(leaf.into());
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

    let res_string = serde_json::to_string(&top).unwrap();
    std::fs::write("out.json", res_string).unwrap();
}

fn build_tree(node: &mut Dir, parts: &Vec<String>, depth: usize, path_full: &str) {
    if depth < parts.len() {
        let item = &parts[depth];

        let dir = match node.find_child(item) {
            Some(d) => d,
            None => {
                let d = Dir::new(item, path_full);
                node.add_child(d);
                match node.find_child(item) {
                    Some(d2) => d2,
                    None => panic!("no child"),
                }
            }
        };
        build_tree(dir, parts, depth + 1, path_full);
    }
}
