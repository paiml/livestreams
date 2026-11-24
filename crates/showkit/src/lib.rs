//! Render facts of one show (`facts.toml`, flat `key = value`) checked against the show shapes
//! (`shapes/show.shacl.ttl`): title set, duration, loudness, true peak, black runs, refusal text.

use std::collections::BTreeMap;

fn unquote(s: &str, line: usize) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                return if blank_or_comment(chars.as_str()) {
                    Ok(out)
                } else {
                    Err(format!("line {line}: trailing characters after quote"))
                };
            }
            '\\' => {
                let Some(esc) = chars.next() else { break };
                if !matches!(esc, '"' | '\\') {
                    out.push('\\');
                }
                out.push(esc);
            }
            _ => out.push(c),
        }
    }
    Err(format!("line {line}: unclosed quote"))
}

fn literal(s: &str, line: usize) -> Result<String, String> {
    match s.split_once('\'') {
        Some((lit, rest)) if blank_or_comment(rest) => Ok(lit.to_string()),
        _ => Err(format!("line {line}: bad literal string")),
    }
}

fn blank_or_comment(rest: &str) -> bool {
    let r = rest.trim();
    r.is_empty() || r.starts_with('#')
}

fn value(v: &str, line: usize) -> Result<String, String> {
    if let Some(s) = v.strip_prefix('"') {
        unquote(s, line)
    } else if let Some(s) = v.strip_prefix('\'') {
        literal(s, line)
    } else {
        Ok(v.split('#').next().unwrap_or("").trim().to_string())
    }
}

/// Parse flat `key = value` lines; `#` comments and blank lines are skipped; strings may be quoted.
///
/// # Errors
/// A non-blank line without `=`.
pub fn parse(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut m = BTreeMap::new();
    for (i, raw) in text.lines().enumerate() {
        let l = raw.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let (k, v) = l
            .split_once('=')
            .ok_or_else(|| format!("line {}: no '='", i + 1))?;
        let k = k.trim();
        if k.is_empty() {
            return Err(format!("line {}: empty key", i + 1));
        }
        let v = v.trim();
        let v = value(v, i + 1)?;
        if m.insert(k.to_string(), v).is_some() {
            return Err(format!("line {}: duplicate key {k}", i + 1));
        }
    }
    Ok(m)
}

fn num(m: &BTreeMap<String, String>, key: &str) -> Result<f64, String> {
    m.get(key)
        .ok_or_else(|| format!("{key}: missing"))?
        .parse::<f64>()
        .map_err(|_| format!("{key}: not a number"))
}

fn range(m: &BTreeMap<String, String>, key: &str, lo: f64, hi: f64, out: &mut Vec<String>) {
    match num(m, key) {
        Ok(v) if (lo..=hi).contains(&v) => {}
        Ok(v) => out.push(format!("{key}: {v} outside {lo}..{hi}")),
        Err(e) => out.push(e),
    }
}

/// Every failed shape rule for these facts; empty means the show passes.
#[must_use]
pub fn check(m: &BTreeMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    if m.get("title").is_none_or(|t| t.trim().is_empty()) {
        out.push("title: not set".to_string());
    }
    range(m, "duration_s", 60.0, 3600.0, &mut out);
    range(m, "loudness_lufs", -24.0, -12.0, &mut out);
    range(m, "true_peak_dbfs", -60.0, -1.0, &mut out);
    range(m, "max_black_run_s", 0.0, 0.5, &mut out);
    match m.get("refusal_text").map(|v| v.parse::<i64>()) {
        Some(Ok(0)) => {}
        Some(Ok(v)) => out.push(format!("refusal_text: {v} outside 0..0")),
        Some(Err(_)) => out.push("refusal_text: not an integer".to_string()),
        None => out.push("refusal_text: missing".to_string()),
    }
    out
}

/// One file: its PASS or FAIL line and whether it passed.
fn report(f: &str) -> (String, bool) {
    let res = std::fs::read_to_string(f)
        .map_err(|e| e.to_string())
        .and_then(|t| parse(&t))
        .map(|m| check(&m));
    match res {
        Ok(v) if v.is_empty() => (format!("PASS {f}"), true),
        Ok(v) => (format!("FAIL {f}: {}", v.join("; ")), false),
        Err(e) => (format!("FAIL {f}: {e}"), false),
    }
}

/// `showkit check FILE...`: prints one PASS or FAIL line per file; returns the exit code.
#[must_use]
pub fn run(args: &[String]) -> i32 {
    let files = args.get(1..).unwrap_or_default();
    if args.first().map(String::as_str) != Some("check") || files.is_empty() {
        eprintln!("usage: showkit check FACTS.toml...");
        return 2;
    }
    let mut code = 0;
    for f in files {
        let (line, ok) = report(f);
        println!("{line}");
        if !ok {
            code = 1;
        }
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "# facts\ntitle = \"Demo\"\nduration_s = 198.5\nloudness_lufs = -15.2\n\
true_peak_dbfs = -2.1\nmax_black_run_s = 0.17\nrefusal_text = 0\n";

    fn facts(s: &str) -> BTreeMap<String, String> {
        parse(s).unwrap()
    }

    #[test]
    fn parse_rejects_empty_duplicate_and_unclosed() {
        assert!(parse("= x\n").unwrap_err().contains("empty key"));
        assert!(parse("a = 1\na = 2\n").unwrap_err().contains("duplicate"));
        assert!(parse("title = \"Demo\n").unwrap_err().contains("unclosed"));
    }

    #[test]
    fn parse_strips_inline_comments_and_unescapes_quotes() {
        let m = facts(
            "title = \"Say \\\"Hi\\\" #1\" # quoted + comment\n\
             slash = \"C:\\\\\" # escaped backslash before quote\n\
             other = \"a\\nb\" # non-quote escape preserved\n\
             duration_s = 198.5 # unquoted + comment\n",
        );
        assert_eq!(m["title"], "Say \"Hi\" #1");
        assert_eq!(m["slash"], "C:\\");
        assert_eq!(m["other"], "a\\nb");
        assert_eq!(m["duration_s"], "198.5");
        assert!(parse("title = \"\\\"\n").unwrap_err().contains("unclosed"));
        assert!(parse("title = \"\\").unwrap_err().contains("unclosed"));
        assert!(parse("title = \"a\" \"b\"\n")
            .unwrap_err()
            .contains("trailing"));
    }

    #[test]
    fn refusal_text_must_be_integer_zero_and_present() {
        let f = facts(&GOOD.replace("refusal_text = 0", "refusal_text = 0.0"));
        assert!(check(&f).iter().any(|e| e.contains("not an integer")));
        let g = facts(&GOOD.replace("refusal_text = 0\n", ""));
        assert!(check(&g).iter().any(|e| e.contains("missing")));
    }

    #[test]
    fn good_facts_pass() {
        assert!(check(&facts(GOOD)).is_empty());
    }

    #[test]
    fn parse_rejects_a_line_without_equals() {
        assert!(parse("title Demo").unwrap_err().contains("line 1"));
    }

    #[test]
    fn each_rule_fails_on_its_own() {
        for (from, to, want) in [
            ("title = \"Demo\"", "title = \"\"", "title: not set"),
            ("duration_s = 198.5", "duration_s = 5", "duration_s"),
            (
                "loudness_lufs = -15.2",
                "loudness_lufs = -30",
                "loudness_lufs",
            ),
            (
                "true_peak_dbfs = -2.1",
                "true_peak_dbfs = 0",
                "true_peak_dbfs",
            ),
            (
                "max_black_run_s = 0.17",
                "max_black_run_s = 6",
                "max_black_run_s",
            ),
            ("refusal_text = 0", "refusal_text = 2", "refusal_text"),
        ] {
            let bad = GOOD.replace(from, to);
            let v = check(&facts(&bad));
            assert!(v.iter().any(|e| e.contains(want)), "{want}: {v:?}");
        }
    }

    #[test]
    fn missing_and_non_numeric_values_fail() {
        let v = check(&facts("title = x\nduration_s = abc\n"));
        assert!(v.iter().any(|e| e == "duration_s: not a number"));
        assert!(v.iter().any(|e| e == "loudness_lufs: missing"));
        assert!(check(&facts(&GOOD.replace("title = \"Demo\"\n", "")))
            .contains(&"title: not set".to_string()));
    }

    #[test]
    fn run_reports_pass_fail_and_usage() {
        let d = std::env::temp_dir().join(format!("showkit-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let ok = d.join("ok.toml");
        let bad = d.join("bad.toml");
        let junk = d.join("junk.toml");
        std::fs::write(&ok, GOOD).unwrap();
        std::fs::write(&bad, GOOD.replace("refusal_text = 0", "refusal_text = 1")).unwrap();
        std::fs::write(&junk, "nonsense").unwrap();
        let s = |p: &std::path::Path| p.display().to_string();
        let c = "check".to_string();
        assert_eq!(run(&[c.clone(), s(&ok)]), 0);
        assert_eq!(run(&[c.clone(), s(&ok), s(&bad)]), 1);
        assert_eq!(run(&[c.clone(), s(&junk)]), 1);
        assert_eq!(run(&[c.clone(), s(&d.join("absent.toml"))]), 1);
        assert_eq!(run(&[c]), 2);
        assert_eq!(run(&["other".to_string(), s(&ok)]), 2);
        std::fs::remove_dir_all(&d).unwrap();
    }
}

#[cfg(test)]
mod parse_edge_tests {
    use super::parse;

    #[test]
    fn quote_in_key_or_value_does_not_hide_comment() {
        assert_eq!(parse("t = foo\"bar # c\n").unwrap()["t"], "foo\"bar");
        assert_eq!(parse("a\"b = 1 # c\n").unwrap()["a\"b"], "1");
        assert_eq!(parse("a\" = \"x\" # c\n").unwrap()["a\""], "x");
    }

    #[test]
    fn single_quoted_literal_keeps_hash() {
        assert_eq!(parse("t = 'C# live' # c\n").unwrap()["t"], "C# live");
        assert!(parse("t = 'open\n").is_err());
    }
}

#[cfg(test)]
mod literal_tests {
    use super::{check, parse};

    #[test]
    fn empty_single_quoted_title_fails_check() {
        let m = parse("title = ''\n").unwrap();
        assert_eq!(m["title"], "");
        assert!(!check(&m).is_empty());
    }
}
