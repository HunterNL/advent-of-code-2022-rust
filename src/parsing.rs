/// Extends `char::is_ascii_digit` with `'-'` to easily select negative numbers
fn is_number_char(char: &char) -> bool {
    char.is_ascii_digit() || char == &'-'
}

pub fn consume_number_from_char_iter<T>(iter: &mut T) -> i32
where
    T: Iterator<Item = char>,
{
    let chars: String = iter
        .skip_while(|char| !is_number_char(char))
        .take_while(is_number_char)
        .collect();

    chars.parse().expect("Chars to parse into numbers")
}

pub fn consume_when<T, P, I>(iter: &mut T, predicate: &P) -> Vec<I>
where
    T: Iterator<Item = I>,
    P: Fn(&I) -> bool,
{
    iter.skip_while(|i| !predicate(i))
        .take_while(predicate)
        .collect()
}

// pub fn chunk_by<T, P, I, R>(iter: &mut T, mut predicate: P) -> TakeWhile<SkipWhile<T, P>, P>
// where
//     T: Iterator<Item = I>,
//     P: FnMut(&I) -> bool,
// {
//     let mut i2 = iter.skip_while(|i| !predicate_1(i));

//     i2.take_while(predicate_2)
// }
