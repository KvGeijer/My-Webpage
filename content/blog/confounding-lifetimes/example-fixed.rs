use std::cell::RefCell;
use std::collections::HashMap;

type Value = i64;

struct Environment<'a> {
    map: RefCell<HashMap<String, Value>>,
    next: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    fn declare(&self, variable: String, value: Value) {
        self.map.borrow_mut().insert(variable, value);
    }

    fn nest(&'a self) -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
            next: Some(self),
        }
    }
}

fn main() {
    let mut base = Environment {
        map: RefCell::new(HashMap::new()),
        next: None,
    };

    {
        let nested = base.nest();
    }
    base.declare("c".to_string(), 13);
}
