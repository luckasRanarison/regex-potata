# Regex-potata

A basic regex engine, built as a practical application of automata theory, implements an E-NFA using Thompson construction and BFS for NFA simulation.

## Usage

```rust
use regex_potata::Regex;

fn main() {
    let re = Regex::new("hello (w|w)orld!*").unwrap();
    let result = re.test("hello world!!!");

    println!("{}", result); // true

    let re = Regex::new(r#"(?<day>\d{2})-(?<month>\d{2})-(?<year>\d{4})"#).unwrap();
    let captures = re.captures("07-01-2024").unwrap();

    println!("{:?}", captures.get_name("day"));
    println!("{:?}", captures.get_name("month"));
    println!("{:?}", captures.get_name("year"));

    let re = Regex::new("(T|t)h(e|(e|o)se)").unwrap();
    let matches = re.find_all("the These those The");

    println!("{:?}", matches);
}
```

## TODOs

- [x] Basic regex `foo` `(bar)` `|` `.`
- [x] Quantifiers `+` `?` `*` `{x}` `{x,y}` `{x,}`
- [x] Character classes `[a-z]` `[^x]` `\d` `\D` `\w` `\W` `\s` `\S`
- [x] Captures `(foo)` `(:?bar)` `(?<named>foo)`
- [ ] Anchors `^` `$`
- [ ] NFA visualizer
