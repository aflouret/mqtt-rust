// Precondicion: filter y topic_name son validos
pub fn filter_matches_topic(filter: &str, topic_name: &str) -> bool {
    let filter_levels: Vec<&str> = filter.split('/').collect();
    let topic_name_levels: Vec<&str> = topic_name.split('/').collect();

    if let Some(index) = filter.split('/').position(|l| l == "#") {
        if index == 0 && topic_name_levels[0].starts_with('$') {
            return false;
        }
        return match_levels(&filter_levels[..index], &topic_name_levels[..index]);
    }

    match_levels(&filter_levels, &topic_name_levels)
}

fn match_levels(filter: &[&str], topic: &[&str]) -> bool {
    if filter.len() != topic.len() {
        return false;
    }

    for (pos, level) in filter.iter().enumerate() {
        if (topic[pos] != *level && *level != "+") || (topic[pos].starts_with('$') && *level == "+")
        {
            return false;
        }
    }
    
    true
}

pub fn topic_filter_is_valid(filter: &str) -> bool {
    let len = filter.len();

    if (filter.contains('#') && filter.find('#') != Some(len - 1))
        || (filter.contains('#') && filter.len() > 1 && !filter[..len - 1].ends_with('/'))
    {
        return false;
    }

    let filter_levels: Vec<&str> = filter.split('/').collect();

    for level in filter_levels.iter() {
        if level.contains('+') && level.len() != 1 {
            return false;
        }
    }

    true
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

    #[test]
    fn test12() {
        assert!(filter_matches_topic("#", "/"))
    }

    #[test]
    fn test13() {
        assert!(filter_matches_topic("/#", "/"))
    }

    #[test]
    fn test14() {
        assert!(filter_matches_topic("#", "/abc/def"))
    }

    #[test]
    fn test15() {
        assert!(filter_matches_topic("/#", "/abc/def"))
    }

    #[test]
    fn test16() {
        assert!(filter_matches_topic("abc/+", "abc/def"))
    }

    #[test]
    fn test17() {
        assert!(filter_matches_topic("abc/+/ghi", "abc/def/ghi"))
    }

    #[test]
    fn test18() {
        assert!(filter_matches_topic("abc/+/ghi/+", "abc/def/ghi/jkl"))
    }

    #[test]
    fn test19() {
        assert!(filter_matches_topic("abc/+/#", "abc/def/ghi/jkl"))
    }

    #[test]
    fn test20() {
        assert!(filter_matches_topic("+/def/+/#", "abc/def/ghi/jkl"))
    }

    #[test]
    fn test21() {
        assert!(filter_matches_topic("+", "abc"))
    }

    #[test]
    fn test22() {
        assert!(filter_matches_topic("/+", "/abc"))
    }

    #[test]
    fn test23() {
        assert!(filter_matches_topic("+/+", "/abc"))
    }

    #[test]
    fn test24() {
        assert!(!filter_matches_topic("+", "/abc"))
    }

    #[test]
    fn test25() {
        assert!(!filter_matches_topic("+", "abc/def"))
    }

    #[test]
    fn test26() {
        assert!(!filter_matches_topic("+", "abc/"))
    }

    #[test]
    fn test27() {
        assert!(!filter_matches_topic("#", "$abc/def"))
    }

    #[test]
    fn test28() {
        assert!(!filter_matches_topic("+/def", "$abc/def"))
    }

    #[test]
    fn test29() {
        assert!(filter_matches_topic("$abc/#", "$abc/def/ghi"))
    }

    #[test]
    fn test30() {
        assert!(filter_matches_topic("$abc/def/+", "$abc/def/ghi"))
    }

    // topic_filter_is_valid tests:

    #[test]
    fn test31() {
        assert!(topic_filter_is_valid("abc/#"))
    }

    #[test]
    fn test32() {
        assert!(!topic_filter_is_valid("abc/#/def"))
    }

    #[test]
    fn test33() {
        assert!(!topic_filter_is_valid("abc#"))
    }

    #[test]
    fn test34() {
        assert!(!topic_filter_is_valid("abc#def"))
    }

    #[test]
    fn test35() {
        assert!(!topic_filter_is_valid("#abc"))
    }

    #[test]
    fn test36() {
        assert!(topic_filter_is_valid("#"))
    }

    #[test]
    fn test37() {
        assert!(topic_filter_is_valid("+"))
    }

    #[test]
    fn test38() {
        assert!(topic_filter_is_valid("+/abc/#"))
    }

    #[test]
    fn test39() {
        assert!(!topic_filter_is_valid("abc+"))
    }

    #[test]
    fn test40() {
        assert!(!topic_filter_is_valid("+abc"))
    }

    #[test]
    fn test41() {
        assert!(!topic_filter_is_valid("abc+def"))
    }
}
