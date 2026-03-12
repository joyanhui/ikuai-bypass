use std::str::FromStr;

pub fn normalize_go_style_args(args: &[String]) -> Vec<String> {
    if args.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(args.len());
    out.push(args[0].to_string());

    for a in &args[1..] {
        if let Some(rewritten) = rewrite_single_dash_long(a) {
            out.push(rewritten);
        } else {
            out.push(a.to_string());
        }
    }

    out
}

pub fn rewrite_single_dash_long(arg: &str) -> Option<String> {
    if !arg.starts_with('-') || arg.starts_with("--") {
        return None;
    }

    const LONGS: [&str; 4] = ["exportPath", "tag", "login", "isIpGroupNameAddRandomSuff"];

    for name in LONGS {
        let prefix = format!("-{}", name);
        if arg == prefix {
            return Some(format!("--{}", name));
        }
        let eq_prefix = format!("-{}=", name);
        if let Some(rest) = arg.strip_prefix(&eq_prefix) {
            return Some(format!("--{}={}", name, rest));
        }
    }

    None
}

pub fn normalize_cron_expr(expr: &str) -> Result<String, String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Err("Cron expression is empty".to_string());
    }

    let parts: Vec<&str> = expr.split_whitespace().collect();
    let mut candidates = Vec::new();
    candidates.push(expr.to_string());
    if parts.len() == 5 {
        candidates.push(format!("0 {}", expr));
        candidates.push(format!("0 {} *", expr));
    }
    if parts.len() == 6 {
        candidates.push(format!("{} *", expr));
    }

    for c in candidates {
        if cron::Schedule::from_str(&c).is_ok() {
            return Ok(c);
        }
    }
    Err("Invalid cron expression".to_string())
}
