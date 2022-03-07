use std::{collections::HashSet, ops::Range};

pub fn naive_search(pattern: &Vec<u8>, text: &Vec<u8>) -> HashSet<Range<usize>> {
    let mut i = 0;
    let mut j = 0;
    let mut sequences = HashSet::new();

    loop {
        if i == pattern.len() {
            let range = (j + i - 1) - (pattern.len() - 1)..j + i;
            sequences.insert(range);
            i = 0;
            j += 1;
        }

        if j >= text.len() || i + j >= text.len() {
            break;
        }

        let byte_to_match = pattern[i];
        let byte = &text[j + i];

        if *byte == byte_to_match {
            i += 1;
        } else {
            i = 0;
            j += 1;
        }
    }
    sequences
}
