use crate::{
    ast::{ClassMember, Node, Range},
    error::ParsingError,
};

type Result<T> = std::result::Result<T, ParsingError>;

pub fn parse_regex(input: &str) -> Result<Node> {
    parse_alternation(input).map(|(result, _)| result)
}

fn parse_alternation(input: &str) -> Result<(Node, &str)> {
    parse_concat(input).and_then(|(lhs, rest)| match rest.get(..1) {
        Some("|") => {
            parse_alternation(&rest[1..]).map(|(rhs, rest)| (Node::alternation(lhs, rhs), rest))
        }
        _ => Ok((lhs, rest)),
    })
}

fn parse_concat(input: &str) -> Result<(Node, &str)> {
    parse_quantifier(input).and_then(|(lhs, rest)| match rest.get(..1) {
        Some("|") | Some(")") | None => Ok((lhs, rest)),
        Some(_) => parse_concat(rest).map(|(rhs, rest)| (Node::concatenation(lhs, rhs), rest)),
    })
}

fn parse_quantifier(input: &str) -> Result<(Node, &str)> {
    parser_atom(input).and_then(|(result, rest)| match rest.get(..1) {
        Some("+") => Ok((Node::plus(result), &rest[1..])),
        Some("*") => Ok((Node::star(result), &rest[1..])),
        Some("?") => Ok((Node::optional(result), &rest[1..])),
        Some("{") => {
            parse_range(&rest[1..]).map(|(range, rest)| (Node::range(result, range), rest))
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
        Some(_) => take_number(input).and_then(|(number, rest)| match (number, rest.get(..1)) {
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
            ')' => Ok((Node::Empty, input)),
            _ => Ok((Node::Character(c), &input[c.len_utf8()..])),
        },
        None => Ok((Node::Empty, input)),
    }
}

fn parse_metachar(input: &str) -> Result<(Node, &str)> {
    match input.chars().next() {
        Some(ch) if needs_escape(ch) => Ok((Node::Character(ch), &input[1..])),
        Some(ch) => get_range_alias(ch)
            .map(|range| (range, &input[1..]))
            .ok_or(ParsingError::InvalidEscapeSequence),
        None => Err(ParsingError::UnexpectedEndOfInput),
    }
}

fn parse_class(input: &str) -> Result<(Node, &str)> {
    let (negate, rest) = match input.get(..1) {
        Some("^") => (true, &input[1..]),
        _ => (false, input),
    };

    parse_class_members(rest).map(|(members, rest)| (Node::class(negate, members), rest))
}

fn parse_class_members(input: &str) -> Result<(Vec<ClassMember>, &str)> {
    parse_class_members_inner(input, Vec::new())
}

fn parse_class_members_inner(
    input: &str,
    acc: Vec<ClassMember>,
) -> Result<(Vec<ClassMember>, &str)> {
    let (ch, is_escaped, rest) = parse_char(input)?;

    if ch == ']' && !is_escaped {
        return Ok((acc, rest));
    }

    if let Some(rest) = rest.strip_prefix('-') {
        let (upper, _, rest) = parse_char(rest)?;
        let acc = vec![acc, vec![ClassMember::Range(ch, upper)]].concat();
        parse_class_members_inner(rest, acc)
    } else {
        let acc = vec![acc, vec![ClassMember::Atom(ch)]].concat();
        parse_class_members_inner(rest, acc)
    }
}

fn parse_char(input: &str) -> Result<(char, bool, &str)> {
    match take_char(input) {
        (Some('\\'), rest) => match take_char(rest) {
            (Some(next), rest) => needs_escape(next)
                .then_some((next, true, rest))
                .ok_or(ParsingError::InvalidEscapeSequence),
            _ => Err(ParsingError::UnexpectedEndOfInput),
        },
        (Some(ch), rest) => Ok((ch, false, rest)),
        _ => Err(ParsingError::UnexpectedEndOfInput),
    }
}

fn parse_group(input: &str) -> Result<(Node, &str)> {
    let (is_capturing, name, rest) = match input.get(..2) {
        Some(":?") => (false, None, &input[2..]),
        Some("?<") => {
            let (name, rest) = take_alphabetic(&input[2..]);

            if name.is_empty() || !rest.starts_with('>') {
                return Err(ParsingError::InvalidCaptureName);
            }

            (true, Some(name), &rest[1..])
        }
        _ => (true, None, input),
    };

    parse_alternation(rest).and_then(|(result, rest)| match rest.get(..1) {
        Some(")") => Ok((Node::group(result, is_capturing, name), &rest[1..])),
        _ => Err(ParsingError::MissingCharacter(')')),
    })
}

fn take_while<'a, P>(predicate: P) -> impl Fn(&'a str) -> (&'a str, &'a str) + 'a
where
    P: Fn(char) -> bool + 'a,
{
    move |input: &'a str| {
        let index = input
            .char_indices()
            .find_map(|(i, c)| (!predicate(c)).then_some(i))
            .unwrap_or(input.len());

        (&input[..index], &input[index..])
    }
}

fn take_number(input: &str) -> Result<(Option<usize>, &str)> {
    let (number, rest) = take_while(|ch| ch.is_ascii_digit())(input);
    let number = number.parse::<usize>().ok();

    Ok((number, rest))
}

fn take_alphabetic(input: &str) -> (&str, &str) {
    take_while(|ch| ch.is_alphabetic())(input)
}

fn take_char(input: &str) -> (Option<char>, &str) {
    match input.chars().next() {
        Some(c) => (Some(c), &input[c.len_utf8()..]),
        None => (None, input),
    }
}

fn needs_escape(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '[' | ']' | '(' | ')' | '{' | '}' | '.' | '?' | '+' | '*' | '-'
    )
}

#[inline]
fn digit_range() -> Vec<ClassMember> {
    vec![ClassMember::Range('0', '9')]
}

#[inline]
fn word_range() -> Vec<ClassMember> {
    vec![
        ClassMember::Range('0', '9'),
        ClassMember::Range('a', 'z'),
        ClassMember::Range('A', 'Z'),
    ]
}

#[inline]
fn whitespace() -> Vec<ClassMember> {
    vec![
        ClassMember::Atom(' '),
        ClassMember::Atom('\t'),
        ClassMember::Atom('\n'),
        ClassMember::Atom('\r'),
        ClassMember::Atom('\x0C'),
        ClassMember::Atom('\x0B'),
    ]
}

fn get_range_alias(ch: char) -> Option<Node> {
    match ch {
        'd' => Some(Node::class(false, digit_range())),
        'D' => Some(Node::class(true, digit_range())),
        'w' => Some(Node::class(false, word_range())),
        'W' => Some(Node::class(true, word_range())),
        's' => Some(Node::class(false, whitespace())),
        'S' => Some(Node::class(true, whitespace())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ClassMember, Node, Range},
        parser::parse_regex,
    };

    #[test]
    fn test_chars() {
        let ast = parse_regex("ok!").unwrap();
        let expected = Node::concatenation(
            Node::Character('o'),
            Node::concatenation(Node::Character('k'), Node::Character('!')),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_unary() {
        let ast = parse_regex("les?").unwrap();
        let expected = Node::concatenation(
            Node::Character('l'),
            Node::concatenation(Node::Character('e'), Node::optional(Node::Character('s'))),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_alternation() {
        let ast = parse_regex("la|le").unwrap();
        let expected = Node::alternation(
            Node::concatenation(Node::Character('l'), Node::Character('a')),
            Node::concatenation(Node::Character('l'), Node::Character('e')),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_group_alternation() {
        let ast = parse_regex("l(a|e)").unwrap();
        let expected = Node::concatenation(
            Node::Character('l'),
            Node::group(
                Node::alternation(Node::Character('a'), Node::Character('e')),
                true,
                None,
            ),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_range_quantifier() {
        let ast = parse_regex("1{2,5}").unwrap();
        let expected = Node::range(Node::Character('1'), Range::new(2, Some(5)));

        assert_eq!(ast, expected);

        let ast = parse_regex("1{5}").unwrap();
        let expected = Node::range(Node::Character('1'), Range::new(5, Some(5)));

        assert_eq!(ast, expected);

        let ast = parse_regex("1{5,}").unwrap();
        let expected = Node::range(Node::Character('1'), Range::new(5, None));

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_character_class() {
        let ast = parse_regex(r#"[bar\\]"#).unwrap();
        let expected = Node::class(
            false,
            vec![
                ClassMember::Atom('b'),
                ClassMember::Atom('a'),
                ClassMember::Atom('r'),
                ClassMember::Atom('\\'),
            ],
        );

        assert_eq!(ast, expected);

        let ast = parse_regex(r#"[^a-zA-Z.]"#).unwrap();
        let expected = Node::class(
            true,
            vec![
                ClassMember::Range('a', 'z'),
                ClassMember::Range('A', 'Z'),
                ClassMember::Atom('.'),
            ],
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_capture_groups() {
        let ast = parse_regex("(foo)bar").unwrap();
        let expected = Node::concatenation(
            Node::group(
                Node::concatenation(
                    Node::Character('f'),
                    Node::concatenation(Node::Character('o'), Node::Character('o')),
                ),
                true,
                None,
            ),
            Node::concatenation(
                Node::Character('b'),
                Node::concatenation(Node::Character('a'), Node::Character('r')),
            ),
        );

        assert_eq!(ast, expected);

        let ast = parse_regex("(:?foo)bar").unwrap();
        let expected = Node::concatenation(
            Node::group(
                Node::concatenation(
                    Node::Character('f'),
                    Node::concatenation(Node::Character('o'), Node::Character('o')),
                ),
                false,
                None,
            ),
            Node::concatenation(
                Node::Character('b'),
                Node::concatenation(Node::Character('a'), Node::Character('r')),
            ),
        );

        assert_eq!(ast, expected);

        let ast = parse_regex("(?<capt>foo)bar").unwrap();
        let expected = Node::concatenation(
            Node::group(
                Node::concatenation(
                    Node::Character('f'),
                    Node::concatenation(Node::Character('o'), Node::Character('o')),
                ),
                true,
                Some("capt"),
            ),
            Node::concatenation(
                Node::Character('b'),
                Node::concatenation(Node::Character('a'), Node::Character('r')),
            ),
        );

        assert_eq!(ast, expected);
    }
}
