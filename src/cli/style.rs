use clap::builder::Styles;
use clap::builder::styling::AnsiColor;

pub fn style() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightGreen.on_default())
        .usage(AnsiColor::BrightGreen.on_default())
        .literal(AnsiColor::BrightCyan.on_default())
        .placeholder(AnsiColor::BrightBlue.on_default())
}
