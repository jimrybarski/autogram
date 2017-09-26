mod wordlist;
use wordlist::LETTER_COUNT_LUT;
extern crate char_iter;
use std::collections::HashMap;
use std::cmp;

type LetterCounts = [u8; 26];

#[derive(Debug, PartialOrd, PartialEq)]
struct UncertainLetter {
    character: char,
    range: Vec<u8>
}

impl UncertainLetter {
    pub fn new(character: char, range: Vec<u8>) -> UncertainLetter {
        UncertainLetter {
            character: character,
            range: range
        }
    }
}

fn char_to_index(c: &char) -> usize {
    // We store counts in 26-member arrays - this maps chars to their position in those arrays.
    // Note that this function is implemented as a long jump table so we can't use it for anything
    // outside of the initialization phase.
    match *c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        'i' => 8,
        'j' => 9,
        'k' => 10,
        'l' => 11,
        'm' => 12,
        'n' => 13,
        'o' => 14,
        'p' => 15,
        'q' => 16,
        'r' => 17,
        's' => 18,
        't' => 19,
        'u' => 20,
        'v' => 21,
        'w' => 22,
        'x' => 23,
        'y' => 24,
        'z' => 25,
        _ => panic!("Invalid character! Only a-z (lowercase) are allowed!")
    }
}

fn build_range(min: u8, max: u8) -> Vec<u8> {
    // Makes a vector with integers that represent possible count values a letter could have
    // We can't use just a min and max since we might be able to rule out some intermediate
    // values, resulting in a discontinuous range
    let mut range = vec![];
    for i in min..max {
        range.push(i);
    }
    range
}

fn count_initial_static_letters(preamble: &str) -> LetterCounts {
    let mut array = [0; 26];
    for c in preamble
             .chars()
             .filter(|x| *x != ' ')
             .chain("and".chars()) {
        let index = char_to_index(&c);
        array[index] += 1;
    }
    array
}

fn add_minimum_s_count(initial_static_counts: &LetterCounts) -> LetterCounts {
    let mut counts = initial_static_counts.clone();
    // If a letter already has a count of two or more, it will have an "s" when its count is
    // enumerated (e.g. "five r's") while something with one will not ("one r")
    let mut s_count = 0;
    for count in initial_static_counts {
        if *count > 1 {
            s_count += 1;
        }
    }
    counts[18] += s_count;
    counts
}

fn determine_solvable_letters(plural_static_counts: &LetterCounts) -> [bool; 26] {
    // solvable letters are those that occur in the initial static words but not in any number word
    let letter_in_number_words = [false, false, false, false, true, true, true, true, true, false, false, true, false, true, true, false, false, true, true, true, true, true, true, true, true, false];
    let mut solvable_letters = [false; 26];
    for (i, (static_count, number_word)) in plural_static_counts.iter().zip(letter_in_number_words.iter()).enumerate() {
        if *static_count > 0 && ! number_word {
            solvable_letters[i] = true;
        }
    }
    solvable_letters
}

fn determine_zero_or_one_only_letters(solvable_letters: &[bool; 26]) -> [bool; 26] {
    // Some letters can only have a value of 0 or 1. These are those letters that don't occur in the
    // initial static words or any number word. If the preamble doesn't contain 'z', for example,
    // you can still have "and one z" at the end. This is really just a way to modulate the number
    // of o's, n's and e's.
    let letter_in_number_words = [false, false, false, false, true, true, true, true, true, false, false, true, false, true, true, false, false, true, true, true, true, true, true, true, true, false];
    let mut zero_or_one_only = [false; 26];
    for (i, (number_word, solvable)) in letter_in_number_words.iter().zip(solvable_letters.iter()).enumerate() {
        if ! number_word && ! solvable {
            zero_or_one_only[i] = true;
        }
    }
    zero_or_one_only
}

fn determine_uncertain_letters(solvable_letters: &[bool; 26], zero_or_one_letters: &[bool; 26]) -> [bool; 26] {
    // Some letters can only have a value of 0 or 1. These are those letters that don't occur in the
    // initial static words or any number word. If the preamble doesn't contain 'z', for example,
    // you can still have "and one z" at the end. This is really just a way to modulate the number
    // of o's, n's and e's.
    let mut uncertain = [false; 26];
    for (i, (solvable, zero_or_one)) in solvable_letters.iter().zip(zero_or_one_letters.iter()).enumerate() {
        if ! solvable && ! zero_or_one {
            uncertain[i] = true;
        }
    }
    uncertain
}

fn initalize_static_alphabet(plural_static_counts: &LetterCounts, solvable_letters: &[bool; 26]) -> [Option<u8>; 26] {
    let mut static_alphabet: [Option<u8>; 26] = [None; 26];
    for (i, (count, solvable)) in plural_static_counts.iter().zip(solvable_letters.iter()).enumerate() {
        if *solvable {
            static_alphabet[i] = Some(*count + 1);
        }
    }
    static_alphabet
}

fn initalize_uncertain_alphabet(uncertain_letters: &[bool; 26]) -> Vec<UncertainLetter> {
    // create a Vec of uncertain letters, with their bounds between 0 and 100. Additionally, we
    // lower the upper bound by figuring out the absolute maximum any letter could be.

    // the maximum possible counts that a letter can get from a single word
    // for example, 'e' occurs 4 times in 'seventeen'
    let number_words_max_char_counts = [
     ('e', 4),
     ('f', 3),
     ('g', 2),
     ('h', 2),
     ('i', 2),
     ('l', 1),
     ('n', 4),
     ('o', 2),
     ('r', 2),
     ('s', 2),
     ('t', 3),
     ('u', 1),
     ('v', 2),
     ('w', 2),
     ('x', 2),
     ('y', 1)
    ].iter().cloned().collect::<HashMap<char, u8>>();

    let uncertain_length: u8 = uncertain_letters.iter().fold(0, |acc, &x| if x { acc + 1} else { acc });

    char_iter::new('a', 'z')
        .zip(uncertain_letters.iter())
        .filter(|&(_, uncertain)| *uncertain)
        .map(|(character, _)| match number_words_max_char_counts.contains_key(&character) {
            // if a static letter, when evaluated, would produce this character, we wouldn't account for it here.
            // later, we will evaluate the static alphabet and raise the uncertain letter limits (both lower and upper)
            true => {
                let factor = number_words_max_char_counts[&character];
                let upper_bound = if character == 's' { factor * uncertain_length + uncertain_length } else { factor * uncertain_length };
                UncertainLetter::new(character, build_range(0, upper_bound + 1))
            },
            false => {
                // I don't think this is possible, but I haven't proven it.
                UncertainLetter::new(character, build_range(0, 100))
            }})
        .collect::<Vec<UncertainLetter>>()
}


fn initialize_zero_or_one_alphabet(zero_or_ones: &[bool; 26]) -> Vec<UncertainLetter> {
    char_iter::new('a', 'z')
        .zip(zero_or_ones.iter())
        .filter(|&(_, zero_or_one)| *zero_or_one)
        .map(|(character, _)| UncertainLetter::new(character, build_range(0, 2)))
        .collect::<Vec<UncertainLetter>>()
}

fn evaluate_static_alphabet(static_alphabet: &[Option<u8>; 26]) -> LetterCounts {
    let mut accumulator = [0u8; 26];
    for (static_letter_index, letter) in static_alphabet.iter().enumerate() {
        if let Some(static_letter_count) = *letter {
            for (acc_index, value) in LETTER_COUNT_LUT[((static_letter_count - 1) as usize * 26) + static_letter_index].iter().enumerate() {
                accumulator[acc_index] += *value;
            }
        }
    }
    accumulator
}

fn add_letter_counts(counts1: &LetterCounts, counts2: &LetterCounts) -> LetterCounts {
    let mut result = [0; 26];
    for (i, (l1, l2)) in counts1.iter().zip(counts2.iter()).enumerate() {
        result[i] = l1 + l2;
    }
    result
}

fn adjust_uncertain_alphabet(uncertain_alphabet: &[UncertainLetter], minimum_counts: &LetterCounts) -> Vec<UncertainLetter> {
    let mut adjusted_alphabet = vec![];
    for letter in uncertain_alphabet {
        let index = char_to_index(&letter.character);
        let minimum = minimum_counts[index];
        let new_range: Vec<u8> = letter.range.iter().map(|&x| cmp::min(x + minimum, 99)).collect();
        adjusted_alphabet.push(UncertainLetter::new(letter.character, new_range));
    }
    adjusted_alphabet
}

fn main() {
    // let preamble = "this bar trivia team name has";
    let preamble = "this sentence contains only";
    let initial_static_counts = count_initial_static_letters(&preamble);
    let plural_static_counts = add_minimum_s_count(&initial_static_counts);

    // exhaustively assign each letter to one of three sets
    // solvable letters are those whose values can be determined in advance
    let solvable = determine_solvable_letters(&plural_static_counts);
    // zero or one only letters do not appear in any number word or in the initial static phrase,
    // so they can only occur 0 or 1 times (like in "one z")
    let zero_or_one_only = determine_zero_or_one_only_letters(&solvable);
    // uncertain letters can potentially have a large number of values
    let uncertain = determine_uncertain_letters(&solvable, &zero_or_one_only);

    // Prove that the three sets are completely distinct.
    for (s, (z, u)) in solvable.iter().zip(zero_or_one_only.iter().zip(uncertain.iter())) {
        assert!(! ((*s && *z) || (*s && *u) || (*z && *u)));
    }

    // contains only letters with guaranteed certain counts.
    let static_alphabet = initalize_static_alphabet(&plural_static_counts, &solvable);
    // create an uncertain alphabet with ranges bounded between 0 and 100
    let uncertain_alphabet = initalize_uncertain_alphabet(&uncertain);
    // put bounds on zero_or_one letters
    let zero_or_one_alphabet = initialize_zero_or_one_alphabet(&zero_or_one_only);

    assert!(26 >= uncertain_alphabet.len() + zero_or_one_alphabet.len());

    // Currently the static alphabet ONLY contains SOLVED letters. So we can evaluate it (that is,
    // turn counts of letters into written words, like a:3 becomes "three a's"
    // We then count up all the resulting letters. With "three a's" for example, we know there MUST
    // be at least two e's. We do this for every solved letter and sum them all together.
    let evaluated_static = evaluate_static_alphabet(&static_alphabet);

    // We can add the initial static counts to the evaluated counts we just generated, since there's
    // no overlap whatsoever (and thus no double counting). These values are therefore the absolute
    // minimum value any letter can hold.
    let minimum_counts = add_letter_counts(&evaluated_static, &initial_static_counts);

    // Taking the absolute lower bounds, we adjust the uncertain alphabet on the both sides.
    // We have to add to both the lower and upper bounds since we don't know if these are just adding
    // to all the counts in the uncertain alphabet
    let adjusted_uncertain_alphabet = adjust_uncertain_alphabet(&uncertain_alphabet, &minimum_counts);

    for letter in adjusted_uncertain_alphabet {
        println!("{}", letter.character);
    }

    // TODO: Refactor! You DON'T need to calculate the uncertain letters in advance!
    // TODO: In fact, they don't need to contain ranges!
    // TODO: It's crazy but true! You just need to iterate over them in a defined order (or maybe not?)
    // TODO: When you pop off an Uncertain Letter, you should decide ON THE FLY what its range is
    // TODO: The formula is, start with the minimum counts, and add the number_words_max_char_counts
    // TODO: Or if it's zero or one, just do 0 and 1
    // TODO: You can probably have an enum like:
    enum Uncertain {
        ZeroOrOne,
        Range(char)
    };
    // TODO: ooh yeah this is definitely the way

}
