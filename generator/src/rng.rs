use anyhow::{bail, Result};
use git2::Time;

#[derive(Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Result<Self> {
        if seed == 0 {
            bail!("seed must be non-zero (xorshift produces only zeros with seed=0)");
        }
        Ok(Self(seed))
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn usize(&mut self, max: usize) -> usize {
        (self.next_u64() % max as u64) as usize
    }

    pub fn pick<'a>(&mut self, items: &'a [&str]) -> &'a str {
        items[self.usize(items.len())]
    }
}

pub const COMMIT_TYPES: &[&str] = &[
    "feat", "fix", "refactor", "perf", "chore", "docs", "ci", "test",
];
pub const WORDS_A: &[&str] = &[
    "update",
    "add",
    "remove",
    "refactor",
    "improve",
    "fix",
    "handle",
    "support",
    "implement",
    "optimize",
];
pub const WORDS_B: &[&str] = &[
    "feature",
    "endpoint",
    "handler",
    "logic",
    "validation",
    "error",
    "check",
    "flow",
    "config",
    "output",
];

pub fn rand_message(rng: &mut Rng, scope: &str) -> String {
    let t = rng.pick(COMMIT_TYPES);
    let bang = if rng.usize(20) == 0 { "!" } else { "" };
    let a = rng.pick(WORDS_A);
    let b = rng.pick(WORDS_B);
    format!("{t}({scope}){bang}: {a} {b}")
}

pub fn rand_time(rng: &mut Rng, now: i64) -> Time {
    let days = rng.usize(365) as i64;
    let hours = rng.usize(24) as i64;
    let mins = rng.usize(60) as i64;
    let offset = days * 86400 + hours * 3600 + mins * 60;
    Time::new(now - offset, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence() {
        let mut a = Rng::new(123).unwrap();
        let mut b = Rng::new(123).unwrap();
        let vals_a: Vec<usize> = (0..20).map(|_| a.usize(1000)).collect();
        let vals_b: Vec<usize> = (0..20).map(|_| b.usize(1000)).collect();
        assert_eq!(vals_a, vals_b);
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = Rng::new(1).unwrap();
        let mut b = Rng::new(2).unwrap();
        let vals_a: Vec<usize> = (0..10).map(|_| a.usize(1000)).collect();
        let vals_b: Vec<usize> = (0..10).map(|_| b.usize(1000)).collect();
        assert_ne!(vals_a, vals_b);
    }

    #[test]
    fn usize_within_range() {
        let mut rng = Rng::new(42).unwrap();
        for _ in 0..200 {
            let v = rng.usize(10);
            assert!(v < 10, "got {v}, expected < 10");
        }
    }

    #[test]
    fn pick_returns_item_from_slice() {
        let items = &["a", "b", "c"];
        let mut rng = Rng::new(99).unwrap();
        for _ in 0..50 {
            let picked = rng.pick(items);
            assert!(items.contains(&picked));
        }
    }

    #[test]
    fn rand_message_format() {
        let mut rng = Rng::new(42).unwrap();
        let msg = rand_message(&mut rng, "core");
        // Must match pattern: type(scope)[!]: word word
        assert!(msg.contains("(core)"), "missing scope in: {msg}");
        assert!(msg.contains(": "), "missing colon-space in: {msg}");

        let colon_pos = msg.find(": ").unwrap();
        let prefix = &msg[..colon_pos];
        // prefix is like "feat(core)" or "fix(core)!"
        let paren = prefix.find('(').unwrap();
        let type_part = &prefix[..paren];
        assert!(
            COMMIT_TYPES.contains(&type_part),
            "unknown commit type '{type_part}' in: {msg}"
        );

        let body = &msg[colon_pos + 2..];
        let words: Vec<&str> = body.split_whitespace().collect();
        assert_eq!(words.len(), 2, "expected two words in body, got: {body}");
        assert!(WORDS_A.contains(&words[0]), "unknown word_a '{}'", words[0]);
        assert!(WORDS_B.contains(&words[1]), "unknown word_b '{}'", words[1]);
    }

    #[test]
    fn rand_time_is_in_the_past() {
        let mut rng = Rng::new(42).unwrap();
        let now = 1_700_000_000i64;
        for _ in 0..50 {
            let t = rand_time(&mut rng, now);
            assert!(t.seconds() <= now);
        }
    }

    #[test]
    fn seed_zero_rejected() {
        let err = Rng::new(0).unwrap_err();
        assert!(
            err.to_string().contains("non-zero"),
            "unexpected error: {err}"
        );
    }
}
