#[derive(Default)]
pub struct TextCharacteristic {
    pub number_of_consonant: i32,
    pub number_of_vowel: i32,
    pub number_of_space: i32,
    pub number_of_special_character: i32,
}

pub fn consonant_count(some_string: &str) -> usize {
    const CONSONANTS: &str = "bcdfghjklmnpqrstvwxyz";
    some_string
        .chars()
        .filter(|c| CONSONANTS.contains(*c))
        .count()
}

pub fn vowel_count(some_string: &str) -> usize {
    const VOWELS: &str = "aeiou";
    some_string.chars().filter(|c| VOWELS.contains(*c)).count()
}

pub fn space_count(some_string: &str) -> usize {
    some_string.chars().filter(|&c| c == ' ').count()
}

pub fn parse_text(string: &str) -> TextCharacteristic {
    let mut textcarac = TextCharacteristic::default();
    textcarac.number_of_consonant = consonant_count(string) as i32;
    textcarac.number_of_space = space_count(string) as i32;
    textcarac.number_of_vowel = vowel_count(string) as i32;
    textcarac.number_of_special_character = string.to_string().chars().count() as i32
        - textcarac.number_of_consonant
        - textcarac.number_of_space
        - textcarac.number_of_vowel;
    return textcarac;
}
