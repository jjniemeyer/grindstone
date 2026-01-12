use crate::models::Category;

/// Validation result with error message
pub type ValidationResult = Result<(), &'static str>;

/// Validate that a session name is not empty
pub fn validate_session_name(name: &str) -> bool {
    !name.trim().is_empty()
}

/// Validate a category name for creation
///
/// Returns Ok(()) if valid, or Err with a message explaining why it's invalid.
pub fn validate_new_category_name(name: &str, existing: &[Category]) -> ValidationResult {
    if name.trim().is_empty() {
        return Err("Category name cannot be empty");
    }
    if existing.iter().any(|c| c.name == name) {
        return Err("Category already exists");
    }
    Ok(())
}

/// Validate a category name for update
///
/// Same as new category validation but allows the category to keep its existing name.
pub fn validate_update_category_name(
    name: &str,
    existing: &[Category],
    current_name: &str,
) -> ValidationResult {
    if name.trim().is_empty() {
        return Err("Category name cannot be empty");
    }
    // Allow keeping the same name, but not colliding with other categories
    if name != current_name && existing.iter().any(|c| c.name == name) {
        return Err("Category already exists");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn make_categories() -> Vec<Category> {
        vec![
            Category {
                id: None,
                name: "Work".to_string(),
                color: Color::Red,
            },
            Category {
                id: None,
                name: "Study".to_string(),
                color: Color::Blue,
            },
        ]
    }

    #[test]
    fn test_validate_session_name_empty() {
        assert!(!validate_session_name(""));
        assert!(!validate_session_name("   "));
        assert!(!validate_session_name("\t\n"));
    }

    #[test]
    fn test_validate_session_name_valid() {
        assert!(validate_session_name("Work session"));
        assert!(validate_session_name("a"));
        assert!(validate_session_name("  trimmed  ")); // Has content after trim
    }

    #[test]
    fn test_validate_new_category_name_empty() {
        let categories = make_categories();
        assert_eq!(
            validate_new_category_name("", &categories),
            Err("Category name cannot be empty")
        );
        assert_eq!(
            validate_new_category_name("   ", &categories),
            Err("Category name cannot be empty")
        );
    }

    #[test]
    fn test_validate_new_category_name_duplicate() {
        let categories = make_categories();
        assert_eq!(
            validate_new_category_name("Work", &categories),
            Err("Category already exists")
        );
    }

    #[test]
    fn test_validate_new_category_name_valid() {
        let categories = make_categories();
        assert_eq!(validate_new_category_name("Exercise", &categories), Ok(()));
    }

    #[test]
    fn test_validate_update_category_same_name() {
        let categories = make_categories();
        // Should allow keeping same name
        assert_eq!(
            validate_update_category_name("Work", &categories, "Work"),
            Ok(())
        );
    }

    #[test]
    fn test_validate_update_category_new_unique_name() {
        let categories = make_categories();
        // Should allow changing to a new unique name
        assert_eq!(
            validate_update_category_name("Exercise", &categories, "Work"),
            Ok(())
        );
    }

    #[test]
    fn test_validate_update_category_name_collision() {
        let categories = make_categories();
        // Should not allow changing to an existing name
        assert_eq!(
            validate_update_category_name("Study", &categories, "Work"),
            Err("Category already exists")
        );
    }
}
