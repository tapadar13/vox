use async_trait::async_trait;

use crate::{error::VoxResult, ports::TextRefiner};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormatterConfig {
    pub trim_filler_words: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FormatterPipeline {
    config: FormatterConfig,
}

impl FormatterPipeline {
    pub fn new(config: FormatterConfig) -> Self {
        Self { config }
    }

    pub fn format(&self, input: &str) -> String {
        let cleaned = strip_model_artifacts(input);
        let normalized = normalize_whitespace(&cleaned);
        let punctuated = normalize_punctuation_spacing(&normalized);
        let without_fillers = if self.config.trim_filler_words {
            trim_fillers(&punctuated)
        } else {
            punctuated
        };
        capitalize_sentences(without_fillers.trim())
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleTextRefiner {
    pipeline: FormatterPipeline,
}

impl RuleTextRefiner {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            pipeline: FormatterPipeline::new(config),
        }
    }
}

#[async_trait]
impl TextRefiner for RuleTextRefiner {
    async fn refine(&self, text: &str) -> VoxResult<String> {
        Ok(self.pipeline.format(text))
    }
}

fn strip_model_artifacts(input: &str) -> String {
    const ARTIFACTS: [&str; 6] = [
        "[BLANK_AUDIO]",
        "[MUSIC]",
        "[Music]",
        "(music)",
        "[APPLAUSE]",
        "[Applause]",
    ];

    ARTIFACTS.iter().fold(input.to_owned(), |text, artifact| {
        text.replace(artifact, "")
    })
}

fn normalize_whitespace(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pending_space = false;

    for character in input.chars() {
        if character.is_whitespace() {
            pending_space = !output.is_empty();
        } else {
            if pending_space {
                output.push(' ');
            }
            output.push(character);
            pending_space = false;
        }
    }
    output
}

fn normalize_punctuation_spacing(input: &str) -> String {
    let characters: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());

    for (index, character) in characters.iter().copied().enumerate() {
        if matches!(character, ',' | '.' | '!' | '?' | ':' | ';') {
            while output.ends_with(' ') {
                output.pop();
            }
            output.push(character);

            let next = characters.get(index + 1).copied();
            let decimal_point = character == '.'
                && output
                    .chars()
                    .rev()
                    .nth(1)
                    .is_some_and(|value| value.is_numeric())
                && next.is_some_and(|value| value.is_numeric());
            if !decimal_point
                && next.is_some_and(|value| {
                    !value.is_whitespace() && !matches!(value, ',' | '.' | '!' | '?' | ':' | ';')
                })
            {
                output.push(' ');
            }
        } else if character.is_whitespace() {
            if !output.is_empty() && !output.ends_with(' ') {
                output.push(' ');
            }
        } else {
            output.push(character);
        }
    }

    output.trim().to_owned()
}

fn trim_fillers(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|word| {
            let normalized = word
                .trim_matches(|character: char| !character.is_alphabetic())
                .to_lowercase();
            !matches!(normalized.as_str(), "um" | "uh" | "erm" | "hmm")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_sentences(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut capitalize_next = true;
    let characters: Vec<char> = input.chars().collect();

    for (index, character) in characters.iter().copied().enumerate() {
        if capitalize_next && character.is_alphabetic() {
            output.extend(character.to_uppercase());
            capitalize_next = false;
        } else {
            output.push(character);
        }

        let decimal_point = character == '.'
            && index > 0
            && characters[index - 1].is_numeric()
            && characters
                .get(index + 1)
                .is_some_and(|value| value.is_numeric());
        if matches!(character, '.' | '!' | '?') && !decimal_point {
            capitalize_next = true;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_whitespace_and_punctuation() {
        let formatter = FormatterPipeline::default();
        assert_eq!(
            formatter.format("  hello   world ,this is vox !  "),
            "Hello world, this is vox!"
        );
    }

    #[test]
    fn capitalizes_each_sentence_without_damaging_unicode() {
        let formatter = FormatterPipeline::default();
        assert_eq!(
            formatter.format("hello. नमस्ते दुनिया! bonjour? مرحبا"),
            "Hello. नमस्ते दुनिया! Bonjour? مرحبا"
        );
    }

    #[test]
    fn preserves_decimal_numbers() {
        let formatter = FormatterPipeline::default();
        assert_eq!(
            formatter.format("version 3.14 is ready"),
            "Version 3.14 is ready"
        );
    }

    #[test]
    fn removes_known_non_speech_artifacts() {
        let formatter = FormatterPipeline::default();
        assert_eq!(formatter.format("[BLANK_AUDIO] hello (music)"), "Hello");
    }

    #[test]
    fn filler_trimming_is_explicitly_opt_in() {
        let default = FormatterPipeline::default();
        let trimmed = FormatterPipeline::new(FormatterConfig {
            trim_filler_words: true,
        });

        assert_eq!(default.format("um this is useful"), "Um this is useful");
        assert_eq!(trimmed.format("um this is useful"), "This is useful");
    }
}
