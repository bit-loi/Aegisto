use crate::types::StringMatch;

/// Extract ASCII printable strings of at least `min_len` bytes from `data`.
pub fn extract_strings(data: &[u8], min_len: usize) -> Vec<StringMatch> {
    let mut results = Vec::new();
    let mut current = Vec::new();
    let mut start_offset = 0;

    for (i, &byte) in data.iter().enumerate() {
        if (0x20..=0x7e).contains(&byte) {
            if current.is_empty() {
                start_offset = i;
            }
            current.push(byte);
        } else {
            if current.len() >= min_len {
                // Safety: we only pushed ASCII bytes (0x20..=0x7e), so from_utf8 is fine.
                let s = String::from_utf8_lossy(&current).into_owned();
                results.push(StringMatch {
                    offset: start_offset,
                    value: s,
                });
            }
            current.clear();
        }
    }

    // Handle string that extends to end of data.
    if current.len() >= min_len {
        let s = String::from_utf8_lossy(&current).into_owned();
        results.push(StringMatch {
            offset: start_offset,
            value: s,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_extraction() {
        let data = b"hello\x00world\x00";
        let strs = extract_strings(data, 3);
        assert_eq!(strs.len(), 2);
        assert_eq!(strs[0].value, "hello");
        assert_eq!(strs[0].offset, 0);
        assert_eq!(strs[1].value, "world");
        assert_eq!(strs[1].offset, 6);
    }

    #[test]
    fn skips_short_strings() {
        let data = b"ab\x00abcd\x00";
        let strs = extract_strings(data, 4);
        assert_eq!(strs.len(), 1);
        assert_eq!(strs[0].value, "abcd");
    }

    #[test]
    fn string_at_eof() {
        let data = b"abcdefgh";
        let strs = extract_strings(data, 4);
        assert_eq!(strs.len(), 1);
        assert_eq!(strs[0].value, "abcdefgh");
    }
}
