mod wordlist;
use wordlist::LETTER_COUNT_LUT;
extern crate char_iter;
//extern crate rayon;
//use rayon::prelude::*;

type LetterCounts = [u8; 26];

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


fn has_low_counts(static_counts: &[Option<u8>; 26], calculated: &LetterCounts) -> bool {
    for (calc, count) in calculated.iter().zip(static_counts.iter()) {
        if let Some(c) = *count {
            if c < *calc {
                return true;
            }
        }
    }
    false
}

//fn parallel_solve(static_alphabet: &[Option<u8>; 26],
//         uncertain_alphabet: &[(char, u8)],
//         calculated_counts: LetterCounts) {
//    if let Some((new_static_letter, new_uncertain_alphabet)) = uncertain_alphabet.split_first() {
//        let &(character, max_count) = new_static_letter;
//        let index = char_to_index(&character);
//        let current_count = calculated_counts[index];
//        let range: Vec<u8> = (current_count..max_count + current_count + 1).collect();
//        range.par_iter().for_each(|count|
//        {
//            if character == 'e' {
//                println!("e: {}", count);
//            }
//            let mut new_static_alphabet = static_alphabet.clone();
//            new_static_alphabet[index] = Some(*count);
//            let new_calculated_counts = if *count > 0 {
//                let evaluated_counts = LETTER_COUNT_LUT[((*count as usize - 1) * 26 + index)];
//                add_letter_counts(&evaluated_counts, &calculated_counts)
//            } else {
//                calculated_counts
//            };
//            if ! has_low_counts(&new_static_alphabet, &new_calculated_counts) {
//                let check_zero_counts = count == 0;
//                solve(&new_static_alphabet, &new_uncertain_alphabet, new_calculated_counts, forbidden_counts);
//            }
//        });
//    } else {
//        for (calculated, count) in calculated_counts.iter().zip(static_alphabet.iter()) {
//            if let Some(c) = *count {
//                if c != *calculated {
//                    return;
//                }
//            }
//        }
//        println!("VALID SOLUTION!");
//        println!("{:?}", static_alphabet);
//    }
//}


fn solve(static_alphabet: &[Option<u8>; 26],
         uncertain_alphabet: &[(char, UncertainLetter)],
         calculated_counts: &LetterCounts) {
    if let Some((new_static_letter, new_uncertain_alphabet)) = uncertain_alphabet.split_first() {
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
        let index = char_to_index(&character);
        let current_count = calculated_counts[index];
        'count_loop: for count in current_count..max_count + current_count + 1 {
            if character == 'e' {
                println!("e: {}", count);
            }
            let mut new_static_alphabet = static_alphabet.clone();
            new_static_alphabet[index] = Some(count);
            let evaluated_counts = LETTER_COUNT_LUT[((count as usize - 1) * 26 + index)];
            let new_calculated_counts = if count > 0 {
                add_letter_counts(&evaluated_counts, calculated_counts)
            } else {
                *calculated_counts
            };

            if has_low_counts(&new_static_alphabet, &new_calculated_counts) {
                continue;
            }

            // See if we've not trimmed the upper bounds enough
            for &(index, count) in UNCERTAIN_MAX_COUNTS_PER_WORD.iter() {
                if let Some(static_count) = new_static_alphabet[index] {
                    // different rules if it's in "one" b/c of zero_or_one letters
                    let calc_count = calculated_counts[index];
                    let max_possible_count = if index == 4 || index == 13 || index == 14 {
                        // TODO: BUG! Not taking into account current_count? We have to add numbers we've definitely committed to
                        // but we're not... I don't even know anymore
                        // HAVE SOME DISCIPLINE! YOU HAVE A LOT OF WORK TO DO ON SUNDAY!
                        count * uncertain_remaining + zero_or_one_remaining
                    } else {
                        count * uncertain_remaining
                    };
                    if static_count > max_possible_count {
                        continue 'count_loop;
                    }
                }
            }
            solve(&new_static_alphabet, &new_uncertain_alphabet, &new_calculated_counts);
        }
    } else {
        // We might have a solution! Let's check and see if it's correct.
        validate_solution();
        println!("VALID SOLUTION!");
        println!("{:?}", static_alphabet);
    }
}

fn validate_solution(calculated_counts: &LetterCounts, static_alphabet: &[Option<u8>; 26]) {
    // TODO: Make this go, write a unit test for it, maybe add the preamble and solve it the old-fashioned way
    for (calculated, count) in calculated_counts.iter().zip(static_alphabet.iter()) {
        if let Some(c) = *count {
            if c != *calculated {
                return;
            }
        }
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

    let zero_or_one_count = zero_or_one_chars.len() as u8;

    let uncertain_alphabet: Vec<(char, UncertainLetter)> = vec![
        // TODO: explain these devil magic numbers
        ('e', UncertainLetter::Variable(4*16+zero_or_one_count)),
        ('t', UncertainLetter::Variable(3*15)),
        ('o', UncertainLetter::Variable(2*14+zero_or_one_count)),
        ('i', UncertainLetter::Variable(2*13)),
        ('n', UncertainLetter::Variable(4*12+zero_or_one_count)),
        ('s', UncertainLetter::Variable(2*11)),
        ('r', UncertainLetter::Variable(2*10)),
        ('h', UncertainLetter::Variable(2*9)),
        ('l', UncertainLetter::Variable(1*8)),
        ('u', UncertainLetter::Variable(1*7)),
        ('f', UncertainLetter::Variable(3*6)),
        ('y', UncertainLetter::Variable(1*5)),
        ('w', UncertainLetter::Variable(2*5)),
        ('g', UncertainLetter::Variable(2*4)),
        ('v', UncertainLetter::Variable(2*3)),
        ('x', UncertainLetter::Variable(2*2))
    ].into_iter()
        .chain(zero_or_one_chars
            .iter()
            .map(|&c| (c, UncertainLetter::ZeroOrOne))
        )
        .collect();
    // println!("{:?}", uncertain_alphabet);
    // kick off the search
    solve(&static_alphabet, &uncertain_alphabet, &minimum_counts);
}
