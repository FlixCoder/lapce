use std::sync::Arc;

use floem::{
    event::EventListener,
    reactive::{ReadSignal, SignalGet, SignalWith},
    style::Style,
    view::View,
    views::{container, container_box, label, list, stack, tab, Decorators},
};

use crate::{
    app::clickable_icon,
    config::{color::LapceColor, icon::LapceIcons, LapceConfig},
    file_explorer::view::file_explorer_panel,
    window_tab::WindowTabData,
};

use super::{
    debug_view::debug_panel,
    global_search_view::global_search_panel,
    kind::PanelKind,
    plugin_view::plugin_panel,
    position::{PanelContainerPosition, PanelPosition},
    problem_view::problem_panel,
    terminal_view::terminal_panel,
};

pub fn panel_container_view(
    window_tab_data: Arc<WindowTabData>,
    position: PanelContainerPosition,
) -> impl View {
    let panel = window_tab_data.panel.clone();
    let config = window_tab_data.common.config;
    let is_bottom = position.is_bottom();
    stack(|| {
        (
            panel_picker(window_tab_data.clone(), position.first()),
            panel_view(window_tab_data.clone(), position.first()),
            panel_view(window_tab_data.clone(), position.second()),
            panel_picker(window_tab_data.clone(), position.second()),
        )
    })
    .on_event(EventListener::DragEnter, move |_| {
        // println!("drag enter");
        true
    })
    .on_event(EventListener::DragLeave, move |_| {
        // println!("drag leave");
        true
    })
    .on_event(EventListener::Drop, move |_| true)
    .style(move || {
        let size = panel.size.with(|s| match position {
            PanelContainerPosition::Left => s.left,
            PanelContainerPosition::Bottom => s.bottom,
            PanelContainerPosition::Right => s.right,
        });
        let is_maximized = panel.panel_bottom_maximized(true);
        let config = config.get();
        Style::BASE
            .apply_if(!panel.is_container_shown(&position, true), |s| s.hide())
            .apply_if(position == PanelContainerPosition::Bottom, |s| {
                s.width_pct(100.0)
                    .apply_if(!is_maximized, |s| {
                        s.border_top(1.0).height_px(size as f32)
                    })
                    .apply_if(is_maximized, |s| s.flex_grow(1.0))
            })
            .apply_if(position == PanelContainerPosition::Left, |s| {
                s.border_right(1.0)
                    .width_px(size as f32)
                    .height_pct(100.0)
                    .background(*config.get_color(LapceColor::PANEL_BACKGROUND))
            })
            .apply_if(position == PanelContainerPosition::Right, |s| {
                s.border_left(1.0)
                    .width_px(size as f32)
                    .height_pct(100.0)
                    .background(*config.get_color(LapceColor::PANEL_BACKGROUND))
            })
            .apply_if(!is_bottom, |s| s.flex_col())
            .border_color(*config.get_color(LapceColor::LAPCE_BORDER))
            .color(*config.get_color(LapceColor::PANEL_FOREGROUND))
    })
}

fn panel_view(
    window_tab_data: Arc<WindowTabData>,
    position: PanelPosition,
) -> impl View {
    let panel = window_tab_data.panel.clone();
    let panels = move || {
        panel
            .panels
            .with(|p| p.get(&position).cloned().unwrap_or_default())
    };
    let active_fn = move || {
        panel
            .styles
            .with(|s| s.get(&position).map(|s| s.active).unwrap_or(0))
    };
    tab(
        active_fn,
        panels,
        |p| *p,
        move |kind| {
            let view = match kind {
                PanelKind::Terminal => container_box(|| {
                    Box::new(terminal_panel(window_tab_data.clone()))
                }),
                PanelKind::FileExplorer => container_box(|| {
                    Box::new(file_explorer_panel(window_tab_data.clone(), position))
                }),
                PanelKind::SourceControl => {
                    container_box(|| Box::new(blank_panel()))
                }
                PanelKind::Plugin => container_box(|| {
                    Box::new(plugin_panel(window_tab_data.clone(), position))
                }),
                PanelKind::Search => container_box(|| {
                    Box::new(global_search_panel(window_tab_data.clone(), position))
                }),
                PanelKind::Problem => container_box(|| {
                    Box::new(problem_panel(window_tab_data.clone(), position))
                }),
                PanelKind::Debug => container_box(|| {
                    Box::new(debug_panel(window_tab_data.clone(), position))
                }),
            };
            view.style(|| Style::BASE.size_pct(100.0, 100.0))
        },
    )
    .style(move || {
        Style::BASE
            .size_pct(100.0, 100.0)
            .apply_if(!panel.is_position_shown(&position, true), |s| s.hide())
    })
}

pub fn panel_header(
    header: String,
    config: ReadSignal<Arc<LapceConfig>>,
) -> impl View {
    container(|| label(move || header.clone())).style(move || {
        Style::BASE
            .padding_horiz_px(10.0)
            .padding_vert_px(6.0)
            .width_pct(100.0)
            .background(*config.get().get_color(LapceColor::EDITOR_BACKGROUND))
    })
}

fn panel_picker(
    window_tab_data: Arc<WindowTabData>,
    position: PanelPosition,
) -> impl View {
    let panel = window_tab_data.panel.clone();
    let panels = panel.panels;
    let config = window_tab_data.common.config;
    let is_bottom = position.is_bottom();
    let is_first = position.is_first();
    list(
        move || {
            panel
                .panels
                .with(|panels| panels.get(&position).cloned().unwrap_or_default())
        },
        |p| *p,
        move |p| {
            let window_tab_data = window_tab_data.clone();
            let icon = match p {
                PanelKind::Terminal => LapceIcons::TERMINAL,
                PanelKind::FileExplorer => LapceIcons::FILE_EXPLORER,
                PanelKind::SourceControl => LapceIcons::SCM,
                PanelKind::Plugin => LapceIcons::EXTENSIONS,
                PanelKind::Search => LapceIcons::SEARCH,
                PanelKind::Problem => LapceIcons::PROBLEM,
                PanelKind::Debug => LapceIcons::DEBUG_ALT,
            };
            let is_active = {
                let window_tab_data = window_tab_data.clone();
                move || {
                    if let Some((active_panel, _)) = window_tab_data
                        .panel
                        .active_panel_at_position(&position, true)
                    {
                        active_panel == p
                    } else {
                        false
                    }
                }
            };
            container(|| {
                stack(|| {
                    (
                        clickable_icon(
                            || icon,
                            move || {
                                window_tab_data.toggle_panel_visual(p);
                            },
                            || false,
                            || false,
                            config,
                        )
                        .draggable()
                        .on_event(EventListener::DragStart, move |_| {
                            println!("drag start on icon");
                            true
                        })
                        .dragging_style(move || {
                            let config = config.get();
                            Style::BASE
                                .border(1.0)
                                .border_radius(6.0)
                                .border_color(
                                    *config.get_color(LapceColor::LAPCE_BORDER),
                                )
                                .padding_px(6.0)
                                .background(
                                    *config.get_color(LapceColor::PANEL_BACKGROUND),
                                )
                        })
                        .style(|| Style::BASE.padding_px(1.0)),
                        label(|| "".to_string()).style(move || {
                            Style::BASE
                                .absolute()
                                .size_pct(100.0, 100.0)
                                .apply_if(!is_bottom && is_first, |s| {
                                    s.margin_top_px(2.0)
                                })
                                .apply_if(!is_bottom && !is_first, |s| {
                                    s.margin_top_px(-2.0)
                                })
                                .apply_if(is_bottom && is_first, |s| {
                                    s.margin_left_px(-2.0)
                                })
                                .apply_if(is_bottom && !is_first, |s| {
                                    s.margin_left_px(2.0)
                                })
                                .apply_if(is_active(), |s| {
                                    s.apply_if(!is_bottom && is_first, |s| {
                                        s.border_bottom(2.0)
                                    })
                                    .apply_if(!is_bottom && !is_first, |s| {
                                        s.border_top(2.0)
                                    })
                                    .apply_if(is_bottom && is_first, |s| {
                                        s.border_left(2.0)
                                    })
                                    .apply_if(is_bottom && !is_first, |s| {
                                        s.border_right(2.0)
                                    })
                                })
                                .border_color(*config.get().get_color(
                                    LapceColor::LAPCE_TAB_ACTIVE_UNDERLINE,
                                ))
                        }),
                    )
                })
            })
            .style(|| Style::BASE.padding_px(6.0))
        },
    )
    .style(move || {
        Style::BASE
            .border_color(*config.get().get_color(LapceColor::LAPCE_BORDER))
            .apply_if(
                panels.with(|p| {
                    p.get(&position).map(|p| p.is_empty()).unwrap_or(true)
                }),
                |s| s.hide(),
            )
            .apply_if(is_bottom, |s| s.flex_col())
            .apply_if(is_bottom && is_first, |s| s.border_right(1.0))
            .apply_if(is_bottom && !is_first, |s| s.border_left(1.0))
            .apply_if(!is_bottom && is_first, |s| s.border_bottom(1.0))
            .apply_if(is_bottom && !is_first, |s| s.border_top(1.0))
    })
}

fn blank_panel() -> impl View {
    label(|| "blank".to_string())
}
