//! Shared name-mangling helpers.
//!
//! `to_snake_case` converts a KeBNF rule name (PascalCase, or ALL_CAPS for
//! lexical terminals) into the snake_case identifier tree-sitter expects.
//! This is the single source of truth for that conversion: both the
//! tree-sitter emitter and the mapping.json generator must agree on it, or
//! mapping.json ends up describing rule names the emitter never actually
//! writes (see CHANGELOG for the terminal-name mangling bug this fixes).

/// Convert a KeBNF rule name to a tree-sitter-style snake_case identifier.
///
/// All-caps names (lexical terminals like `NAME`, `WHITE_SPACE`,
/// `BASIC_NAME`) are lowercased directly rather than split at each
/// uppercase letter, since every character being uppercase does not mean
/// every character starts a new word. Mixed-case PascalCase names (e.g.
/// `AttributeUsage`) are split into words at each uppercase letter
/// boundary, as usual.
pub fn to_snake_case(name: &str) -> String {
    if name.chars().all(|c| c.is_uppercase() || c == '_') {
        return name.to_lowercase();
    }
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_case_splits_on_uppercase() {
        assert_eq!(to_snake_case("FooBar"), "foo_bar");
        assert_eq!(to_snake_case("AttributeUsage"), "attribute_usage");
    }

    #[test]
    fn all_caps_terminals_lowercase_without_splitting() {
        assert_eq!(to_snake_case("FOO"), "foo");
        assert_eq!(to_snake_case("NAME"), "name");
        assert_eq!(to_snake_case("BASIC_NAME"), "basic_name");
        assert_eq!(to_snake_case("UNRESTRICTED_NAME"), "unrestricted_name");
        assert_eq!(to_snake_case("WHITE_SPACE"), "white_space");
        assert_eq!(to_snake_case("CONJUGATES"), "conjugates");
        assert_eq!(to_snake_case("SPECIALIZES"), "specializes");
    }
}
