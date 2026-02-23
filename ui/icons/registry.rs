use dioxus::prelude::*;

use super::set_a::{
    ClockIcon, CodeIcon, DatabaseIcon, GitBranchIcon, GitForkIcon, GlobeIcon, MailIcon, MergeIcon,
    MessageSquareIcon, RepeatIcon, ShuffleIcon, SparklesIcon, WebhookIcon,
};
use super::set_b::{CheckCircleIcon, FileOutputIcon, LoaderIcon, SendIcon};
use super::set_c::{ServerIcon, XIcon, ZapIcon};

/// Returns an icon component by name, or a fallback circle if unknown.
#[allow(clippy::missing_errors_doc)]
pub fn icon_by_name(name: &str, class: String) -> Element {
    match name {
        "server" => rsx! { ServerIcon { class } },
        "zap" => rsx! { ZapIcon { class } },
        "webhook" => rsx! { WebhookIcon { class } },
        "clock" => rsx! { ClockIcon { class } },
        "mail" => rsx! { MailIcon { class } },
        "globe" => rsx! { GlobeIcon { class } },
        "database" => rsx! { DatabaseIcon { class } },
        "shuffle" => rsx! { ShuffleIcon { class } },
        "code" => rsx! { CodeIcon { class } },
        "sparkles" => rsx! { SparklesIcon { class } },
        "git-branch" => rsx! { GitBranchIcon { class } },
        "git-fork" => rsx! { GitForkIcon { class } },
        "repeat" => rsx! { RepeatIcon { class } },
        "merge" => rsx! { MergeIcon { class } },
        "message-square" => rsx! { MessageSquareIcon { class } },
        "send" => rsx! { SendIcon { class } },
        "file-output" => rsx! { FileOutputIcon { class } },
        "loader" => rsx! { LoaderIcon { class } },
        "check-circle" => rsx! { CheckCircleIcon { class } },
        "x" => rsx! { XIcon { class } },
        _ => rsx! {
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                class: "{class}",
                circle { cx: "12", cy: "12", r: "10" }
            }
        },
    }
}
