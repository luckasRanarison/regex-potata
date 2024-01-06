# Regex-potata

A basic regex engine, built as a practical application of automata theory, implements an E-NFA using Thompson construction and BFS for NFA simulation.

## Usage

```rust
use regex_potata::regex::Regex;

fn main() {
  let re = Regex::new("Hello (W|w)orld!*").unwrap();
  let result = re.test("Hello world!!!");

  println!("{}", result); // true
}
```

## TODOs

- [x] Basic regex `foo` `(bar)` `|` `.`
- [x] Quantifiers `+` `?` `*` `{x}` `{x,y}` `{x,}`
- [x] Character classes `[a-z]` `[^x]` `\d` `\D` `\w` `\W` `\s` `\S`
- [x] Captures `(foo)` `(:?bar)` `(?<named>foo)`
- [ ] Anchors `^` `$`
- [ ] NFA visualizer
