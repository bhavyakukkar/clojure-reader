// cargo test --example data -F data
use clojure_reader::{
  data::Datum,
  edn,
  parse::{Node, NodeKind},
};
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Person {
  name: String,
  age: u8,
}

impl fmt::Display for Person {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Person({}, {})", self.name, self.age)
  }
}

fn main() {
  let mut reader = edn::Reader::new();

  reader.add_reader("person", |node| {
    // Expect a vector of two elements - a symbol (name) and an integer (age)
    if let NodeKind::Vector(nodes, _) = node.kind
      && let [
        Node { kind: NodeKind::Symbol(name), span: _, .. },
        Node { kind: NodeKind::Int(age), span: _, .. },
      ] = nodes.as_slice()
    {
      let person = Person { name: name.to_string(), age: *age as u8 };
      Ok(edn::Edn::Data(
        // requires that `Person` impls `Debug`, `Display`, `Clone`, `PartialEq`, `Eq`,
        // `PartialOrd`, `Ord` & `Hash`
        Datum::new(person),
      ))
    } else {
      panic!("unexpected")
    }
  });

  let source = r#" #person [John 34] "#;
  let edn::Edn::Data(data) = reader.read_string(source).unwrap() else { panic!("unexpected") };
  let person: Person = *data.downcast().unwrap();

  assert_eq!(person.name, "John");
  assert_eq!(person.age, 34);
}

#[test]
fn run() {
  main();
}
