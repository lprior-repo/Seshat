#![cfg_attr(test, allow(warnings))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(
    clippy::assigning_clones,
    clippy::branches_sharing_code,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cloned_instead_of_copied,
    clippy::default_trait_access,
    clippy::derive_partial_eq_without_eq,
    clippy::double_must_use,
    clippy::float_cmp,
    clippy::imprecise_flops,
    clippy::iter_on_single_items,
    clippy::manual_let_else,
    clippy::manual_midpoint,
    clippy::manual_range_contains,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::must_use_candidate,
    clippy::needless_pass_by_ref_mut,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::ptr_arg,
    clippy::redundant_else,
    clippy::ref_option,
    clippy::suboptimal_flops,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_result_map_or_else,
    clippy::unnecessary_to_owned,
    clippy::unnecessary_wraps,
    dead_code,
    unused_imports,
    unused_variables
)]
#![cfg_attr(
    test,
    allow(
        clippy::duplicated_attributes,
        clippy::expect_used,
        clippy::ignore_without_reason,
        clippy::panic,
        clippy::similar_names,
        clippy::unwrap_used
    )
)]
#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

use dioxus::prelude::*;
mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli_persistence;
pub mod core;
mod export;
mod geometry;
mod history;
mod hooks;
mod icons;
mod layout;
mod mutation;
mod test_utils;
mod ui;

use crate::app::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(|info| {
            let msg = format!("PANIC: {}", info);
            web_sys::console::error_1(&msg.into());
        }));
    }

    let builder =
        dioxus::LaunchBuilder::new().with_context(server_only! { ServeConfig::builder() });

    builder.launch(App);
}
