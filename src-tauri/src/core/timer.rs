fn normalize(c: char) -> char {
    match c {
        'O' | 'o' | 'D' | 'Q' => '0',
        'l' | 'I' | '|' | 'i' | '!' => '1',
        'Z' | 'z' => '2',
        'S' | 's' => '5',
        'B' => '8',
        ';' | '.' | ',' => ':',
        other => other,
    }
}

fn separators_only(c: char) -> char {
    c
}

pub fn parse_clock(text: &str) -> Option<u32> {
    scan(text, separators_only, false).or_else(|| scan(text, normalize, true))
}

pub fn two_digits(text: &str) -> Option<String> {
    let d: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            'O' | 'o' | 'D' | 'Q' => '0',
            'l' | 'I' | '|' | 'i' | '!' => '1',
            'Z' | 'z' => '2',
            'S' | 's' => '5',
            'B' => '8',
            other => other,
        })
        .collect();
    (d.len() == 2 && d.chars().all(|c| c.is_ascii_digit())).then_some(d)
}

pub fn parse_clock_digits(text: &str) -> Option<u32> {
    let digits: String = text
        .chars()
        .map(|c| match c {
            'O' | 'o' | 'D' | 'Q' => '0',
            'l' | 'I' | '|' | 'i' | '!' => '1',
            'Z' | 'z' => '2',
            'S' | 's' => '5',
            'B' => '8',
            other => other,
        })
        .filter(|c| !c.is_whitespace())
        .collect();
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n = |a: usize| digits[a..a + 2].parse::<u32>().ok();
    if digits.len() != 6 {
        return None;
    }
    let (h, m, s) = (n(0)?, n(2)?, n(4)?);
    (m < 60 && s < 60).then_some(h * 3600 + m * 60 + s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Default)]
pub struct ClockFilter {
    last: Option<(u32, f64)>,
    streak: u32,
}

impl ClockFilter {
    pub fn reset(&mut self) {
        self.last = None;
        self.streak = 0;
    }

    pub fn feed(&mut self, value: u32, now_s: f64, dir: Direction) -> u32 {
        let consistent = match self.last {
            Some((prev, at)) => {
                let dt = (now_s - at).max(0.0);
                let expected = match dir {
                    Direction::Up => prev as f64 + dt,
                    Direction::Down => prev as f64 - dt,
                };
                let advanced = match dir {
                    Direction::Up => value > prev,
                    Direction::Down => value < prev,
                };
                advanced && (value as f64 - expected).abs() <= 2.0 + dt * 0.1
            }
            None => false,
        };
        self.streak = if consistent { self.streak + 1 } else { 1 };
        self.last = Some((value, now_s));
        self.streak
    }
}

fn scan(text: &str, map: fn(char) -> char, squash_spaces: bool) -> Option<u32> {
    let chars: Vec<char> = text.chars().map(map).filter(|c| !squash_spaces || !c.is_whitespace()).collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() || (i > 0 && chars[i - 1].is_ascii_digit()) {
            i += 1;
            continue;
        }
        let mut groups: Vec<u32> = Vec::new();
        let mut j = i;
        loop {
            let start = j;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            let len = j - start;
            if len == 0 || len > 2 || (!groups.is_empty() && len != 2) {
                groups.clear();
                break;
            }
            let n: String = chars[start..j].iter().collect();
            groups.push(n.parse().ok()?);
            if j < chars.len() && chars[j] == ':' && groups.len() < 3 {
                j += 1;
            } else {
                break;
            }
        }
        if groups.len() >= 2 && groups.iter().skip(1).all(|&g| g < 60) {
            return Some(match groups.as_slice() {
                [m, s] => m * 60 + s,
                [h, m, s] => h * 3600 + m * 60 + s,
                _ => unreachable!(),
            });
        }
        i = j.max(i + 1);
    }
    None
}

pub fn fmt_clock(secs: u32) -> String {
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_shapes() {
        assert_eq!(parse_clock("04:50"), Some(290));
        assert_eq!(parse_clock("4:5O"), Some(290));
        assert_eq!(parse_clock("Server time 1:02:03"), Some(3723));
        assert_eq!(parse_clock("  0l : 3O "), Some(90));
        assert_eq!(parse_clock("5.00"), Some(300));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_clock(""), None);
        assert_eq!(parse_clock("Halloween"), None);
        assert_eq!(parse_clock("12:75"), None);
        assert_eq!(parse_clock("123:45"), None);
        assert_eq!(parse_clock("4:5"), None);
    }

    #[test]
    fn strict_parse_wins_over_lenient() {
        assert_eq!(parse_clock("S1 2:30"), Some(150));
    }

    #[test]
    fn filter_needs_readings_that_follow_real_time() {
        let mut f = ClockFilter::default();
        assert_eq!(f.feed(288, 0.0, Direction::Up), 1);
        assert_eq!(f.feed(289, 1.0, Direction::Up), 2);
        assert_eq!(f.feed(290, 2.0, Direction::Up), 3);
        assert_eq!(f.feed(300, 5.0, Direction::Up), 1);
        assert_eq!(f.feed(301, 6.0, Direction::Up), 2);
    }

    #[test]
    fn filter_rejects_one_off_misreads() {
        let mut f = ClockFilter::default();
        f.feed(110, 0.0, Direction::Up);
        f.feed(111, 1.0, Direction::Up);
        assert_eq!(f.feed(290, 2.0, Direction::Up), 1);
        assert_eq!(f.feed(113, 3.0, Direction::Up), 1);
        assert_eq!(f.feed(114, 4.0, Direction::Up), 2);
    }

    #[test]
    fn frozen_value_never_builds_a_streak() {
        let mut f = ClockFilter::default();
        assert_eq!(f.feed(830, 0.0, Direction::Up), 1);
        assert_eq!(f.feed(830, 1.1, Direction::Up), 1);
        assert_eq!(f.feed(830, 2.2, Direction::Up), 1);
    }

    #[test]
    fn version_line_is_not_a_clock_when_timer_is_read() {
        assert_eq!(parse_clock("00:04:02\nVersion 13.50"), Some(242));
        assert_eq!(parse_clock("Version 13.50"), Some(830));
    }

    #[test]
    fn filter_counts_down() {
        let mut f = ClockFilter::default();
        f.feed(15, 0.0, Direction::Down);
        assert_eq!(f.feed(14, 1.0, Direction::Down), 2);
        assert_eq!(f.feed(20, 2.0, Direction::Down), 1);
    }

    #[test]
    fn two_digit_groups() {
        assert_eq!(two_digits("oo"), Some("00".into()));
        assert_eq!(two_digits(" 1O "), Some("10".into()));
        assert_eq!(two_digits("5"), None);
        assert_eq!(two_digits("567"), None);
        assert_eq!(two_digits("ab"), None);
    }

    #[test]
    fn digits_without_colons_parse_strictly() {
        assert_eq!(parse_clock_digits("oo 10 54"), Some(654));
        assert_eq!(parse_clock_digits("001054"), Some(654));
        assert_eq!(parse_clock_digits("oo 54"), None);
        assert_eq!(parse_clock_digits("04 34"), None);
        assert_eq!(parse_clock_digits("00 75 10"), None);
        assert_eq!(parse_clock_digits("12345"), None);
        assert_eq!(parse_clock_digits("Version 13 50"), None);
        assert_eq!(parse_clock_digits(""), None);
    }

    #[test]
    fn formats() {
        assert_eq!(fmt_clock(290), "4:50");
        assert_eq!(fmt_clock(3723), "1:02:03");
    }
}
