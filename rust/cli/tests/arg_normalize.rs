use ikb_cli::normalize_cron_expr;
use ikb_cli::{normalize_go_style_args, rewrite_single_dash_long};

#[test]
fn rewrite_single_dash_long_known_flags() {
    assert_eq!(
        rewrite_single_dash_long("-exportPath").as_deref(),
        Some("--exportPath")
    );
    assert_eq!(
        rewrite_single_dash_long("-exportPath=/tmp").as_deref(),
        Some("--exportPath=/tmp")
    );
    assert_eq!(rewrite_single_dash_long("-r").as_deref(), None);
}

#[test]
fn normalize_keeps_argv0() {
    let args = vec!["ikuai-bypass".to_string(), "-exportPath=/tmp".to_string()];
    let out = normalize_go_style_args(&args);
    assert_eq!(out[0], "ikuai-bypass");
    assert_eq!(out[1], "--exportPath=/tmp");
}

#[test]
fn normalize_cron_accepts_5_fields_from_go_config() {
    let expr = normalize_cron_expr("0 7 * * *").expect("normalize");
    assert!(!expr.is_empty());
}
