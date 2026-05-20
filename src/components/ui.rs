//! Shared UI primitives — buttons, headings, labels and surface cards.
//!
//! Every component accepts an extra `class: String` that is appended to the
//! base utility classes so callers can layer on spacing / sizing without
//! forking the component.

use dioxus::prelude::*;

/// Visual variants for [`Button`].
///
/// - [`Primary`](Self::Primary) — solid blue, the default call-to-action.
/// - [`Secondary`](Self::Secondary) — bordered white, lower-emphasis action.
/// - [`Ghost`](Self::Ghost) — text-only neutral, sits inside dense card UIs.
/// - [`Link`](Self::Link) — text-only blue, mimics an `<a>` link.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    /// Tailwind classes that render this variant.
    fn classes(self) -> &'static str {
        match self {
            Self::Primary => {
                "bg-blue-600 text-white rounded-lg px-4 py-2 hover:bg-blue-700 text-sm font-medium"
            }
            Self::Secondary => {
                "bg-white text-gray-800 border border-gray-300 rounded-lg px-4 py-2 hover:bg-gray-100 text-sm font-medium"
            }
            Self::Ghost => {
                "text-sm text-gray-500 hover:text-gray-700 font-medium bg-white border-0"
            }
            Self::Link => {
                "text-sm text-blue-600 hover:text-blue-800 font-medium bg-white border-0"
            }
        }
    }
}

/// Reusable button with one of [`ButtonVariant`]'s visual styles.
///
/// ```ignore
/// Button {
///     variant: ButtonVariant::Primary,
///     onclick: move |_| save(),
///     "Save"
/// }
/// ```
///
/// Extra utility classes can be merged in via `class`. `onclick` is optional —
/// a no-op handler is used when omitted.
#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let merged = format!("{} {}", variant.classes(), class);
    rsx! {
        button {
            class: "{merged}",
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}

/// Heading sizes accepted by [`Heading`].
///
/// Maps 1:1 to the HTML element rendered (`H1` → `<h1>`, etc.), so semantic
/// document structure follows visual hierarchy.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum HeadingLevel {
    #[default]
    H1,
    H2,
    H3,
}

/// Page or section heading.
///
/// Named `Heading` rather than `Title` to avoid colliding with
/// `dioxus::prelude::Title`, which sets the document `<title>`.
///
/// ```ignore
/// Heading { level: HeadingLevel::H2, "Settings" }
/// ```
#[component]
pub fn Heading(
    #[props(default)] level: HeadingLevel,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let base = match level {
        HeadingLevel::H1 => "text-2xl font-bold text-gray-800",
        HeadingLevel::H2 => "text-xl font-bold text-gray-800",
        HeadingLevel::H3 => "text-lg font-semibold text-gray-800",
    };
    let merged = format!("{base} {class}");
    match level {
        HeadingLevel::H1 => rsx! { h1 { class: "{merged}", {children} } },
        HeadingLevel::H2 => rsx! { h2 { class: "{merged}", {children} } },
        HeadingLevel::H3 => rsx! { h3 { class: "{merged}", {children} } },
    }
}

/// Small caption or form-label text.
///
/// Set `muted: true` for very low-emphasis text (e.g. inline metadata like
/// shortened IDs); the default colour is the regular secondary grey.
#[component]
pub fn Label(
    #[props(default)] class: String,
    #[props(default = false)] muted: bool,
    children: Element,
) -> Element {
    let color = if muted {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let merged = format!("text-xs {color} {class}");
    rsx! { span { class: "{merged}", {children} } }
}

/// Surface container — white background, rounded corners, shadow and border.
///
/// Pass `hover_lift: true` for the dashboard-card hover effect (shadow grows,
/// card translates up). Use `class` to override padding / width.
#[component]
pub fn Card(
    #[props(default)] class: String,
    #[props(default = false)] hover_lift: bool,
    children: Element,
) -> Element {
    let base = "flex flex-col gap-3 p-5 bg-white rounded-xl shadow-md border border-gray-200";
    let hover = if hover_lift {
        "hover:shadow-xl hover:-translate-y-1 transition-all"
    } else {
        ""
    };
    let merged = format!("{base} {hover} {class}");
    rsx! { div { class: "{merged}", {children} } }
}
