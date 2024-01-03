use crate::{
    ast::{Class, ClassMember, Node, Range},
    error::ParsingError,
};

type Result<T> = std::result::Result<T, ParsingError>;

fn parse_regex(input: &str) -> Result<Node> {
    parse_alternation(input).map(|(result, _)| result)
}

fn parse_alternation(input: &str) -> Result<(Node, &str)> {
    parse_concat(input).and_then(|(lhs, rest)| match rest.get(..1) {
        Some("|") => parse_alternation(&rest[1..])
            .map(|(rhs, rest)| (Node::Alternation(Box::new(lhs), Box::new(rhs)), rest)),
        _ => Ok((lhs, rest)),
    })
}

fn parse_concat(input: &str) -> Result<(Node, &str)> {
    parse_quantifier(input).and_then(|(lhs, rest)| match rest.get(..1) {
        Some("|") | Some(")") | None => Ok((lhs, rest)),
        Some(_) => parse_concat(rest)
            .map(|(rhs, rest)| (Node::Concatenation(Box::new(lhs), Box::new(rhs)), rest)),
    })
}

fn parse_quantifier(input: &str) -> Result<(Node, &str)> {
    parser_atom(input).and_then(|(result, rest)| match rest.get(..1) {
        Some("+") => Ok((Node::Plus(Box::new(result)), &rest[1..])),
        Some("*") => Ok((Node::Star(Box::new(result)), &rest[1..])),
        Some("?") => Ok((Node::Optional(Box::new(result)), &rest[1..])),
        Some("{") => {
            let (range, rest) = parse_range(&rest[1..])?;
            let node = Node::Range {
                inner: Box::new(result),
                range,
            };

            Ok((node, rest))
        }
        _ => Ok((result, rest)),
    })
}

fn parse_range(input: &str) -> Result<(Range, &str)> {
    take_number(input).and_then(|(lower, rest)| match (lower, rest.get(..1)) {
        (Some(lower), Some(",")) => {
            parse_range_upper(&rest[1..]).map(|(upper, rest)| (Range::new(lower, upper), rest))
        }
        (Some(lower), Some("}")) => Ok((Range::new(lower, Some(lower)), &rest[1..])),
        _ => Err(ParsingError::InvalidRangeQuantifier),
    })
}

fn parse_range_upper(input: &str) -> Result<(Option<usize>, &str)> {
    match input.get(..1) {
        Some("}") => Ok((None, &input[1..])),
        Some(_) => take_number(&input).and_then(|(number, rest)| match (number, rest.get(..1)) {
            (Some(number), Some("}")) => Ok((Some(number), &rest[1..])),
            _ => Err(ParsingError::InvalidRangeQuantifier),
        }),
        None => Err(ParsingError::InvalidRangeQuantifier),
    }
}

fn parser_atom(input: &str) -> Result<(Node, &str)> {
    match input.chars().next() {
        Some(c) => match c {
            '(' => parse_group(&input[1..]),
            '[' => parse_class(&input[1..]),
            '\\' => parse_metachar(&input[1..]),
            '.' => Ok((Node::Wildcard, &input[1..])),
            _ => Ok((Node::Character(c), &input[c.len_utf8()..])),
        },
        None => Ok((Node::Empty, &input)),
    }
}

fn parse_metachar(input: &str) -> Result<(Node, &str)> {
    match input.chars().next() {
        Some(ch) if needs_escape(ch) => Ok((Node::Character(ch), &input[1..])),
        _ => Err(ParsingError::InvalidEscapeSequence),
    }
}

fn parse_class(input: &str) -> Result<(Node, &str)> {
    let (negate, rest) = match input.get(..1) {
        Some("^") => (true, &input[1..]),
        _ => (false, input),
    };

    parse_class_members(rest, Vec::new())
        .map(|(members, rest)| (Node::CharacterClass(Class { negate, members }), rest))
}

fn parse_class_members(input: &str, acc: Vec<ClassMember>) -> Result<(Vec<ClassMember>, &str)> {
    todo!()
}

fn parse_group(input: &str) -> Result<(Node, &str)> {
    parse_alternation(input).and_then(|(result, rest)| match rest.get(..1) {
        Some(")") => Ok((Node::Group(Box::new(result)), &rest[1..])),
        _ => Err(ParsingError::MissingCharacter(')')),
    })
}

fn take_number(input: &str) -> Result<(Option<usize>, &str)> {
    let index = input
        .char_indices()
        .find_map(|(i, c)| (!c.is_ascii_digit()).then_some(i))
        .unwrap_or(input.len());
    let number = input[..index].parse::<usize>().ok();

    Ok((number, &input[index..]))
}

fn needs_escape(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '[' | ']' | '(' | ')' | '{' | '}' | '.' | '?' | '+' | '*'
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Node, Range},
        parser_alt::parse_regex,
    };

    #[test]
    fn test_chars() {
        let ast = parse_regex("ok!").unwrap();
        let expected = Node::Concatenation(
            Box::new(Node::Character('o')),
            Box::new(Node::Concatenation(
                Box::new(Node::Character('k')),
                Box::new(Node::Character('!')),
            )),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_unary() {
        let ast = parse_regex("les?").unwrap();
        let expected = Node::Concatenation(
            Box::new(Node::Character('l')),
            Box::new(Node::Concatenation(
                Box::new(Node::Character('e')),
                Box::new(Node::Optional(Box::new(Node::Character('s')))),
            )),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_alternation() {
        let ast = parse_regex("la|le").unwrap();
        let expected = Node::Alternation(
            Box::new(Node::Concatenation(
                Box::new(Node::Character('l')),
                Box::new(Node::Character('a')),
            )),
            Box::new(Node::Concatenation(
                Box::new(Node::Character('l')),
                Box::new(Node::Character('e')),
            )),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_group_alternation() {
        let ast = parse_regex("l(a|e)").unwrap();
        let expected = Node::Concatenation(
            Box::new(Node::Character('l')),
            Box::new(Node::Group(Box::new(Node::Alternation(
                Box::new(Node::Character('a')),
                Box::new(Node::Character('e')),
            )))),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_range_quantifier() {
        let ast = parse_regex("1{2,5}").unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: Range::new(2, Some(5)),
        };

        assert_eq!(ast, expected);

        let ast = parse_regex("1{5}").unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: Range::new(5, Some(5)),
        };

        assert_eq!(ast, expected);

        let ast = parse_regex("1{5,}").unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: Range::new(5, None),
        };

        assert_eq!(ast, expected);
    }
}
