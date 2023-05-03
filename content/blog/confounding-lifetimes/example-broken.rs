use std::collections::HashMap;

type Value = i64;

struct Environment<'a> {
    map: HashMap<String, Value>,
    next: Option<&'a mut Environment<'a>>,
}

impl<'a> Environment<'a> {
    fn declare(&mut self, variable: String, value: Value) {
        self.map.insert(variable, value);
    }

    fn nest<'b>(&'b mut self) -> Environment<'b>
    where
        'a: 'b,
    {
        Environment {
            map: HashMap::new(),
            next: Some(self),
        }
    }
}

fn main() {
    let mut base = Environment {
        map: HashMap::new(),
        next: None,
    };

    {
        let nested = base.nest();
    }
    base.declare("c".to_string(), 13);
}
