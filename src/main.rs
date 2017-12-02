mod wordlist;
use wordlist::LETTER_COUNT_LUT;
mod output;
extern crate char_iter;
extern crate rayon;
use rayon::prelude::*;
use std::env;

type LetterCounts = [u8; 26];

// We need to check whether certain letters are part of the word "one"
static ONE_INDEXES: [usize; 3] = [14, 13, 4];


#[derive(Debug)]
enum UncertainLetter {
    Variable(u8),
    ZeroOrOne
}

// The maximum number of counts each unceratin letter can acquire per word
// In order of frequency in English text:         e  t  o  i  n  s  r  h  l  u  f  y  w  g  v  x
static UNCERTAIN_MAX_COUNTS_PER_WORD: [(usize, u8); 16] = [(4,  4),
                                                           (19, 3),
                                                           (14, 2),
                                                           (8,  2),
                                                           (13, 4),
                                                           (18, 2),
                                                           (17, 2),
                                                           (7,  2),
                                                           (11, 1),
                                                           (20, 1),
                                                           (5,  3),
                                                           (24, 1),
                                                           (22, 2),
                                                           (6,  2),
                                                           (21, 2),
                                                           (23, 2)];


fn main() {
    let preamble_vec: Vec<String> = env::args().skip(1).map(|s| String::from(s)).collect();
    let preamble: String = preamble_vec.join(" ");
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

    let zero_or_one_count = zero_or_one_chars.len() as u8;

    let uncertain_alphabet: Vec<(char, UncertainLetter)> = vec![
        // For each letter, there's some maximum number of times it can occur per number word. For
        // example, 't' occurs three times in 'thirty-two'. If all counts are 32, then 't' will
        // occur 3*15 times. We know to use 15, because 't' is the 2nd letter we iterate over, and
        // there are 16 variable letters. As we descend through this fixed-order list, the
        // multiplicative factor shrinks, thus massively reducing the search space. This number does
        // not represent a strict upper bound, but rather, a maximum bound on the additional counts
        // that need to be considered on top of the ones that are required by the static counts. For
        // example, if we have 2 y's, we will have AT LEAST 1 't' (from the "two" in "two y's"), and
        // on top of that we could, in principle, have an additional 45 t's.
        ('e', UncertainLetter::Variable( 4 * 16 + zero_or_one_count )),
    	('t', UncertainLetter::Variable( 3 * 15                     )),
        ('o', UncertainLetter::Variable( 2 * 14 + zero_or_one_count )),
        ('i', UncertainLetter::Variable( 2 * 13                     )),
        ('n', UncertainLetter::Variable( 4 * 12 + zero_or_one_count )),
        ('s', UncertainLetter::Variable( 2 * 11                     )),
        ('r', UncertainLetter::Variable( 2 * 10                     )),
        ('h', UncertainLetter::Variable( 2 *  9                     )),
        ('l', UncertainLetter::Variable(      8                     )),
        ('u', UncertainLetter::Variable(      7                     )),
        ('f', UncertainLetter::Variable( 3 *  6                     )),
        ('y', UncertainLetter::Variable(      5                     )),
        ('w', UncertainLetter::Variable( 2 *  5                     )),
        ('g', UncertainLetter::Variable( 2 *  4                     )),
        ('v', UncertainLetter::Variable( 2 *  3                     )),
        ('x', UncertainLetter::Variable( 2 *  2                     ))
    ].into_iter()
        .chain(zero_or_one_chars
            .iter()
            .map(|&c| (c, UncertainLetter::ZeroOrOne))
        )
        .collect();

    // kick off the search
    solve_parallel(&static_alphabet, &uncertain_alphabet, &minimum_counts, &preamble);
}


fn solve_parallel(static_alphabet: &[Option<u8>; 26],
         uncertain_alphabet: &[(char, UncertainLetter)],
         calculated_counts: &LetterCounts,
         preamble: &str) {
    if let Some((new_static_letter, new_uncertain_alphabet)) = uncertain_alphabet.split_first() {
        let (index, iter_range, uncertain_remaining, zero_or_one_remaining) = setup_descent(new_uncertain_alphabet,
                                                                                            new_static_letter,
                                                                                            calculated_counts);
        iter_range.par_iter().for_each(|&count| {
            let (new_static_alphabet, new_calculated_counts) = descend(static_alphabet, calculated_counts, count, index);
            if ! partial_solution_valid(&new_static_alphabet,
                                        &new_calculated_counts,
                                        uncertain_remaining,
                                        zero_or_one_remaining) {
                return;
            }
            solve(&new_static_alphabet, new_uncertain_alphabet, &new_calculated_counts, preamble);
        });
    }
    else {
        println!("This is not a valid preamble.");
    }
}


fn solve(static_alphabet: &[Option<u8>; 26],
         uncertain_alphabet: &[(char, UncertainLetter)],
         calculated_counts: &LetterCounts,
         preamble: &str) {
    if let Some((new_static_letter, new_uncertain_alphabet)) = uncertain_alphabet.split_first() {
        let (index, iter_range, uncertain_remaining, zero_or_one_remaining) = setup_descent(new_uncertain_alphabet,
                                                                                            new_static_letter,
                                                                                            calculated_counts);
        for count in iter_range {
            let (new_static_alphabet, new_calculated_counts) = descend(static_alphabet, calculated_counts, count, index);
            if ! partial_solution_valid(&new_static_alphabet,
                                        &new_calculated_counts,
                                        uncertain_remaining,
                                        zero_or_one_remaining) {
                continue;
            }
            solve(&new_static_alphabet, new_uncertain_alphabet, &new_calculated_counts, preamble);
        }
    }
    // We might have a solution! Let's check and see if it's correct.
    else if validate_solution(calculated_counts, static_alphabet) {
        let written_solution = output::build_written_solution(static_alphabet, preamble);
        println!("{}", written_solution);
    }
}


fn setup_descent(new_uncertain_alphabet: &[(char, UncertainLetter)],
                 new_static_letter: &(char, UncertainLetter),
                 calculated_counts: &LetterCounts) -> (usize, std::vec::Vec<u8>, u8, u8) {
    let (uncertain_remaining, zero_or_one_remaining) = new_uncertain_alphabet
            .iter()
            .map(|&(_, ref uncertain_letter)| match *uncertain_letter {
                UncertainLetter::Variable(_) => (1u8, 0u8),
                UncertainLetter::ZeroOrOne => (0u8, 1u8)
            })
            .fold((0u8, 0u8), |(var_acc, zoo_acc), (var_inc, zoo_inc)| (var_acc + var_inc, zoo_acc + zoo_inc));

    let &(character, ref uncertain_letter) = new_static_letter;
    let max_count = match *uncertain_letter {
        UncertainLetter::Variable(c) => c,
        UncertainLetter::ZeroOrOne => 1
    };

    let index = char_to_index(&character).unwrap();
    let current_count = calculated_counts[index];
    let iter_range: Vec<u8> = (current_count..max_count + current_count + 1).collect();
    (index, iter_range, uncertain_remaining, zero_or_one_remaining)
}


fn descend(static_alphabet: &[Option<u8>; 26], calculated_counts: &LetterCounts, count: u8, index: usize) -> ([Option<u8>; 26], LetterCounts) {
    let mut new_static_alphabet = *static_alphabet;
    new_static_alphabet[index] = Some(count);
    let new_calculated_counts = if count > 0 {
        let evaluated_counts = LETTER_COUNT_LUT[((count as usize - 1) * 26 + index)];
        add_letter_counts(&evaluated_counts, calculated_counts)
    } else {
        *calculated_counts
    };
    (new_static_alphabet, new_calculated_counts)
}


fn char_to_index(c: &char) -> Option<usize> {
    // We store counts in 26-member arrays - this maps chars to their position in those arrays.
    // Note that this function is implemented as a long jump table so we can't use it for anything
    // outside of the initialization phase.
    match *c {
        'a' => Some(0),
        'b' => Some(1),
        'c' => Some(2),
        'd' => Some(3),
        'e' => Some(4),
        'f' => Some(5),
        'g' => Some(6),
        'h' => Some(7),
        'i' => Some(8),
        'j' => Some(9),
        'k' => Some(10),
        'l' => Some(11),
        'm' => Some(12),
        'n' => Some(13),
        'o' => Some(14),
        'p' => Some(15),
        'q' => Some(16),
        'r' => Some(17),
        's' => Some(18),
        't' => Some(19),
        'u' => Some(20),
        'v' => Some(21),
        'w' => Some(22),
        'x' => Some(23),
        'y' => Some(24),
        'z' => Some(25),
        _ => None
    }
}


fn count_initial_static_letters(preamble: &str) -> LetterCounts {
    let mut array = [0; 26];
    for c in preamble
        .chars()
        .filter(|x| *x != ' ')
        .chain("and".chars()) {
        if let Some(index) = char_to_index(&c) {
            array[index] += 1;
        }
    }
    array
}

fn add_minimum_s_count(initial_static_counts: &LetterCounts) -> LetterCounts {
    let mut counts = *initial_static_counts;
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
    // takes each count, converts it to a written word, and counts each letter. These written words
    // include any plural marker, and the letter itself (for example, in "three f's", there are
    // 2 e's, 1 f, 1 t, and so on).
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
    // Sums counts of letters. We have a current total for the letters observed so far and need
    // to add the counts for one additional letter/count (from wordlist.rs/LETTER_COUNT_LUT)
    let mut result = [0; 26];
    for (i, (l1, l2)) in counts1.iter().zip(counts2.iter()).enumerate() {
        result[i] = l1 + l2;
    }
    result
}

fn has_low_counts(static_counts: &[Option<u8>; 26], calculated: &LetterCounts) -> bool {
    // Determines whether a partial (or complete) solution has any counts that are guaranteed
    // to be too low to be correct. For example, if we've assumed that there are 5 e's and 0 v's,
    // the solution is guaranteed to be invalid because "five" requires at least one v.
    for (calc, count) in calculated.iter().zip(static_counts.iter()) {
        if let Some(c) = *count {
            if c < *calc {
                return true;
            }
        }
    }
    false
}


fn partial_solution_valid(new_static_alphabet: &[Option<u8>; 26],
                          new_calculated_counts: &LetterCounts,
                          uncertain_remaining: u8,
                          zero_or_one_remaining: u8) -> bool {
    if has_low_counts(&new_static_alphabet, &new_calculated_counts) {
        return false;
    }
    if contains_excessive_counts(&new_static_alphabet,
                                 &new_calculated_counts,
                                 uncertain_remaining,
                                 zero_or_one_remaining) {
        return false;
    }
    true
}


fn contains_excessive_counts(new_static_alphabet: &[Option<u8>; 26],
                             new_calculated_counts: &LetterCounts,
                             uncertain_remaining: u8,
                             zero_or_one_remaining: u8) -> bool {
    for &(index, count) in &UNCERTAIN_MAX_COUNTS_PER_WORD {
        if let Some(static_count) = new_static_alphabet[index] {
            // different rules if it's in "one" b/c of zero_or_one letters
            let calc_count = new_calculated_counts[index];
            let max_possible_count = if ONE_INDEXES.contains(&index) {
                count * uncertain_remaining + zero_or_one_remaining + calc_count
            } else {
                count * uncertain_remaining + calc_count
            };
            if static_count > max_possible_count {
                return true;
            }
        }
    }
    false
}


fn validate_solution(calculated_counts: &LetterCounts, static_alphabet: &[Option<u8>; 26]) -> bool {
    for (calculated, count) in calculated_counts.iter().zip(static_alphabet.iter()) {
        if let Some(c) = *count {
            if c != *calculated {
                return false;
            }
        }
    }
    true
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_written_solution() {
        let static_alphabet = [Some(2), None, Some(2), Some(2), Some(28), Some(5), Some(3), Some(8), Some(11), None, None, Some(3), Some(2), Some(13), Some(9), Some(2), None, Some(5), Some(25), Some(23), None, Some(6), Some(10), Some(2), Some(5), Some(1)];
        let written = output::build_written_solution(&static_alphabet, "this sentence employs");
        assert_eq!(written, "this sentence employs two a's, two c's, two d's, twenty-eight e's, five f's, three g's, eight h's, eleven i's, three l's, two m's, thirteen n's, nine o's, two p's, five r's, twenty-five s's, twenty-three t's, six v's, ten w's, two x's, five y's and one z.")
    }

    #[test]
    fn test_validate_solution() {
        let static_alphabet = [Some(2), None, Some(2), Some(2), Some(28), Some(5), Some(3), Some(8), Some(11), None, None, Some(3), Some(2), Some(13), Some(9), Some(2), None, Some(5), Some(25), Some(23), None, Some(6), Some(10), Some(2), Some(5), Some(1)];
        let initial_static_counts = count_initial_static_letters("this sentence employs");
        let letter_counts = evaluate_static_alphabet(&static_alphabet);
        let summed_counts = add_letter_counts(&initial_static_counts, &letter_counts);

        let mut results : Vec<bool> = vec![];
        for (proposed, evaluated) in static_alphabet.iter().zip(summed_counts.iter()) {
            match *proposed {
                Some(p) => { results.push(p == *evaluated); },
                None => { results.push(0 == *evaluated); }
            }
        }
        assert!(! results.contains(&false));
    }

    #[test]
    fn test_validate_solution_invalid() {
        let static_alphabet = [Some(2), None, Some(2), Some(2), Some(28), Some(4), Some(3), Some(8), Some(11), None, None, Some(3), Some(2), Some(13), Some(9), Some(2), None, Some(5), Some(25), Some(23), None, Some(6), Some(10), Some(2), Some(5), Some(1)];
        let initial_static_counts = count_initial_static_letters("this sentence employs");
        let letter_counts = evaluate_static_alphabet(&static_alphabet);
        let summed_counts = add_letter_counts(&initial_static_counts, &letter_counts);

        let mut results : Vec<bool> = vec![];
        for (proposed, evaluated) in static_alphabet.iter().zip(summed_counts.iter()) {
            match *proposed {
                Some(p) => { results.push(p == *evaluated); },
                None => { results.push(0 == *evaluated); }
            }
        }
        assert!(results.contains(&false));
    }

    #[test]
    fn test_validate_solution_invalid2() {
        let static_alphabet = [Some(2), None, Some(2), Some(2), Some(28), Some(5), Some(3), Some(8), Some(11), None, None, Some(3), Some(2), Some(13), Some(9), Some(2), None, Some(5), Some(25), Some(23), None, Some(6), Some(10), Some(2), Some(5), Some(1)];
        let initial_static_counts = count_initial_static_letters("this sentence has");
        let letter_counts = evaluate_static_alphabet(&static_alphabet);
        let summed_counts = add_letter_counts(&initial_static_counts, &letter_counts);

        let mut results : Vec<bool> = vec![];
        for (proposed, evaluated) in static_alphabet.iter().zip(summed_counts.iter()) {
            match *proposed {
                Some(p) => { results.push(p == *evaluated); },
                None => { results.push(0 == *evaluated); }
            }
        }
        assert!(results.contains(&false));
    }
}