use crate::ui::theme::{BG_BASE, BORDER, TEXT_MAIN};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToolbarButtonProps {
    pub test_id: &'static str,
    #[props(default = false)]
    pub disabled: bool,
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
    #[props(default = BG_BASE)]
    pub bg: &'static str,
    #[props(default = TEXT_MAIN)]
    pub color: &'static str,
    #[props(default = BORDER)]
    pub border: &'static str,
    #[props(default)]
    pub title: String,
    #[props(default = 1.0)]
    pub opacity: f32,
    #[props(default = "")]
    pub extra_style: &'static str,
}

#[component]
pub fn ToolbarButton(props: ToolbarButtonProps) -> Element {
    let cursor = if props.disabled {
        "not-allowed"
    } else {
        "pointer"
    };
    let opacity = if props.disabled { 0.4 } else { props.opacity };

    rsx! {
        button {
            "data-testid": props.test_id,
            disabled: props.disabled,
            title: props.title.clone(),
            style: "padding: 6px 10px; cursor: {cursor}; border-radius: 6px; border: 1px solid {props.border}; background: {props.bg}; color: {props.color}; opacity: {opacity}; {props.extra_style}",
            onclick: move |e| props.onclick.call(e),
            {props.children}
        }
    }
}

#[component]
pub fn Divider() -> Element {
    rsx! {
        div { style: "width: 1px; height: 20px; background: {BORDER};" }
    }
}
