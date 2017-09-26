mod wordlist;
use wordlist::LETTER_COUNT_LUT;
use std::collections::HashMap;
use std::cmp;
extern crate char_iter;

type LetterCounts = [u8; 26];


enum Uncertain {
    ZeroOrOne(char),
    Range(char)
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

fn determine_zero_or_one_only_letters(solvable_letters: &[bool; 26]) -> Vec<char> {
    // Some letters can only have a value of 0 or 1. These are those letters that don't occur in the
    // initial static words or any number word. If the preamble doesn't contain 'z', for example,
    // you can still have "and one z" at the end. This is really just a way to modulate the number
    // of o's, n's and e's.
    let letter_in_number_words = [false, false, false, false, true, true, true, true, true, false, false, true, false, true, true, false, false, true, true, true, true, true, true, true, true, false];
    let char_list: Vec<char> = char_iter::new('a', 'z').collect();
    let mut chars = vec![];
    for (i, (number_word, solvable)) in letter_in_number_words.iter().zip(solvable_letters.iter()).enumerate() {
        if ! number_word && ! solvable {
            chars.push(char_list[i]);
        }
    }
    chars
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


fn solve(static_alphabet: &[Option<u8>; 26],
         uncertain_alphabet: &[(char, u8)],
         calculated_counts: LetterCounts) {
    if let Some((new_static_letter, new_uncertain_alphabet)) = uncertain_alphabet.split_first() {
        let &(character, max_count) = new_static_letter;
        let index = char_to_index(&character);
        let current_count = calculated_counts[index];
        for count in current_count..max_count + current_count + 1 {
            let mut new_static_alphabet = static_alphabet.clone();
            new_static_alphabet[index] = Some(count);
            let evaluated_counts = LETTER_COUNT_LUT[((count as usize - 1) * 26 + index)];
            let new_calculated_counts = add_letter_counts(&evaluated_counts, &calculated_counts);
            // TODO: Add validation here
            solve(&new_static_alphabet, &new_uncertain_alphabet, new_calculated_counts);
        }
    } else {
        // TODO: We've assigned values to all letters! Check whether the solution is valid!
    }
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
    let zero_or_one_chars = determine_zero_or_one_only_letters(&solvable);

    // contains only letters with guaranteed certain counts.
    let static_alphabet = initalize_static_alphabet(&plural_static_counts, &solvable);

    // Currently the static alphabet ONLY contains SOLVED letters. So we can evaluate it (that is,
    // turn counts of letters into written words, like a:3 becomes "three a's"
    // We then count up all the resulting letters. With "three a's" for example, we know there MUST
    // be at least two e's. We do this for every solved letter and sum them all together.
    let evaluated_static = evaluate_static_alphabet(&static_alphabet);

    // We can add the initial static counts to the evaluated counts we just generated, since there's
    // no overlap whatsoever (and thus no double counting). These values are therefore the absolute
    // minimum value any letter can hold.
    let minimum_counts = add_letter_counts(&evaluated_static, &initial_static_counts);

    // TODO: Refactor! You DON'T need to calculate the uncertain letters in advance!
    // TODO: In fact, they don't need to contain ranges!
    // TODO: It's crazy but true! You just need to iterate over them in a defined order (or maybe not?)
    // TODO: When you pop off an Uncertain Letter, you should decide ON THE FLY what its range is
    // TODO: The formula is, start with the minimum counts, and add the number_words_max_char_counts
    // TODO: Also evaluate the value being iterated over (the one that was just popped off) and add those
    // TODO: counts to the rest of the uncertain alphabet
    // TODO: If it's zero or one, just do 0 and 1
    // TODO: ooh yeah this is definitely the way

    let uncertain_alphabet: Vec<(char, u8)> = vec![
        // TODO: explain these devil magic numbers
        ('e', 4*16),
        ('t', 3*15),
        ('o', 2*14),
        ('i', 2*13),
        ('n', 4*12),
        ('s', 2*11),
        ('r', 2*10),
        ('h', 2*9),
        ('l', 1*8),
        ('u', 1*7),
        ('f', 3*6),
        ('y', 1*5),
        ('w', 2*5),
        ('g', 2*4),
        ('v', 2*3),
        ('x', 2*2)
    ].into_iter()
     .chain(zero_or_one_chars
            .iter()
            .map(|&c| (c, 1u8))
     )
     .collect();


}
