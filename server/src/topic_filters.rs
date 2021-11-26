// Precondicion: filter y topic_name son validos
pub fn filter_matches_topic(filter: &str, topic_name: &str) -> bool {
    if !filter.contains('#') && !filter.contains('+') {
        return filter == topic_name;
    }

    if filter.contains('#') {
        if filter == "#" {
            return true;
        }

        let filter_prefix = filter.trim_end_matches('#');
        if topic_name == &filter_prefix[..filter_prefix.len()-1] {
            return true;
        }
        return topic_name.starts_with(filter_prefix);
    }

    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01() {
        assert!(filter_matches_topic("abc", "abc"));
    }

    #[test]
    fn test02() {
        assert!(filter_matches_topic("abc/def", "abc/def"));
    }

    #[test]
    fn test03() {
        assert!(!filter_matches_topic("abc/def", "abc/deg"));
    }

    #[test]
    fn test04() {
        assert!(filter_matches_topic("abc/#", "abc/def"));
    }

    #[test]
    fn test05() {
        assert!(filter_matches_topic("abc/#", "abc/def/ghi/jkl"));
    }

    #[test]
    fn test06() {
        assert!(!filter_matches_topic("abc/#", "abd/def"));
    }

    #[test]
    fn test07() {
        assert!(filter_matches_topic("abc/#", "abc"));
    }

    #[test]
    fn test08() {
        assert!(filter_matches_topic("abc/#", "abc/"));
    }

    #[test]
    fn test09() {
        assert!(!filter_matches_topic("abc/#", "abcd"));
    }

    #[test]
    fn test10() {
        assert!(!filter_matches_topic("abc/#", "ab"));
    }

    #[test]
    fn test11() {
        assert!(filter_matches_topic("#", "abc/def"))
    }

    
}