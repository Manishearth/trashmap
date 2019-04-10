use trashmap::TrashSet;
use std::collections::HashMap;
use std::panic;


struct State {
   seen: TrashSet<str>,
   mapping: HashMap<&'static str, Vec<&'static str>>,
}

impl State {
    pub fn step_into(&mut self, entry: &str) {
        let (id, empty) = self.seen.insert_check(entry);
        if !empty {
            panic!("found recursive loop!");
        }
        println!("processing element {}", entry);
        let children = self.mapping.get(entry).cloned().unwrap_or(vec![]);
        println!("\thas children {:?}", children);
        for child in children {
           self.step_into(child);
        }
        self.seen.remove(id);
    }

    fn new(mapping: HashMap<&'static str, Vec<&'static str>>) -> Self {
        Self {
            seen: Default::default(),
            mapping
        }
    }
}


fn test_works() {
    let mut mapping = HashMap::new();
    mapping.insert("foo", vec!["bar", "baz", "quux"]);
    mapping.insert("bar", vec!["baz", "quux"]);
    mapping.insert("quux", vec!["one", "two", "baz"]);
    let mut state = State::new(mapping.clone());
    state.step_into("foo");
    let mut state = State::new(mapping);
    state.step_into("bar");
}

fn test_panics() {
    let mut mapping = HashMap::new();
    mapping.insert("foo", vec!["bar", "baz"]);
    mapping.insert("bar", vec!["baz", "quux"]);
    // this has a cycle and should panic
    mapping.insert("quux", vec!["one", "two", "foo", "baz"]);
    let mut state = State::new(mapping);
    state.step_into("foo");
}

fn main() {
    test_works();
    let result = panic::catch_unwind(|| {
        test_panics();
    });
    assert!(result.is_err())
}
