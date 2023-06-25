use const_format::formatcp;

pub const POPICLI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const POPI_HEADER: &str = formatcp!(
  "◇ popi v{}",
  POPICLI_VERSION,
);
