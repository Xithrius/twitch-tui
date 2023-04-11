use lazy_static::lazy_static;

pub mod start;

lazy_static! {
    pub static ref DASHBOARD_TITLE: Vec<&'static str> = vec![
        "   __           _ __       __          __        _ ",
        "  / /__      __(_) /______/ /_        / /___  __(_)",
        " / __/ | /| / / / __/ ___/ __ \\______/ __/ / / / / ",
        "/ /_ | |/ |/ / / /_/ /__/ / / /_____/ /_/ /_/ / /  ",
        "\\__/ |__/|__/_/\\__/\\___/_/ /_/      \\__/\\__,_/_/   "
    ];
}
