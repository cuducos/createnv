use crate::model::{Block, Comment, SimpleVariable, Variable};

mod model;

fn main() {
    let title = Comment {
        contents: "42".to_string(),
    };
    let description = Some(Comment {
        contents: "Fourty-two".to_string(),
    });
    let variable1 = Box::new(SimpleVariable {
        name: "ANSWER".to_string(),
        input: "42".to_string(),
    }) as Box<dyn Variable>;
    let variable2 = Box::new(SimpleVariable {
        name: "AS_TEXT".to_string(),
        input: "fourty two".to_string(),
    }) as Box<dyn Variable>;
    let variables = vec![variable1, variable2];
    let block = Block {
        title,
        description,
        variables,
    };
    println!("{}", block)
}
