pub fn merge_text(left: &str, right: &str) -> String {
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() {
        return right.to_owned();
    }
    if right.is_empty() {
        return left.to_owned();
    }

    let left_words = left.split_whitespace().collect::<Vec<_>>();
    let right_words = right.split_whitespace().collect::<Vec<_>>();
    let overlap = word_overlap(&left_words, &right_words);
    if overlap == right_words.len() {
        return left.to_owned();
    }

    let suffix = right_words[overlap..].join(" ");
    if suffix.is_empty() {
        left.to_owned()
    } else {
        format!("{left} {suffix}")
    }
}

fn word_overlap(left: &[&str], right: &[&str]) -> usize {
    let maximum = left.len().min(right.len());
    for length in (1..=maximum).rev() {
        let left_start = left.len() - length;
        if left[left_start..]
            .iter()
            .zip(&right[..length])
            .all(|(left_word, right_word)| normalized(left_word) == normalized(right_word))
        {
            return length;
        }
    }
    0
}

fn normalized(word: &str) -> String {
    word.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_exact_overlap_between_chunks() {
        assert_eq!(
            merge_text("hello from the quiet", "the quiet room today"),
            "hello from the quiet room today"
        );
    }

    #[test]
    fn matches_overlap_across_case_and_punctuation() {
        assert_eq!(
            merge_text("We ship today.", "TODAY, after lunch"),
            "We ship today. after lunch"
        );
    }

    #[test]
    fn preserves_non_latin_words() {
        assert_eq!(merge_text("नमस्ते दुनिया", "दुनिया आज"), "नमस्ते दुनिया आज");
    }

    #[test]
    fn keeps_disjoint_phrases() {
        assert_eq!(
            merge_text("first thought", "second thought"),
            "first thought second thought"
        );
    }
}
