use std::{ops::Range, rc::Rc};

use floem::{
    event::EventListener,
    menu::{Menu, MenuItem},
    peniko::kurbo::{Point, Rect, Size},
    reactive::{create_memo, create_rw_signal, RwSignal},
    style::CursorStyle,
    view::View,
    views::{
        container, dyn_container,  img, label, scroll, stack, svg,
        virtual_list, Decorators, VirtualListDirection, VirtualListItemSize,
        VirtualListVector,
    },
};
use indexmap::IndexMap;
use lapce_rpc::plugin::{VoltID, VoltInfo, VoltMetadata};

use super::{kind::PanelKind, position::PanelPosition, view::panel_header};
use crate::{
    app::clickable_icon,
    config::{color::LapceColor, icon::LapceIcons},
    plugin::{AvailableVoltData, InstalledVoltData, PluginData, VoltIcon},
    text_input::text_input,
    window_tab::{Focus, WindowTabData},
};

const VOLT_DEFAULT_PNG: &[u8] = include_bytes!("../../../extra/images/volt.png");

struct IndexMapItems<K, V>(IndexMap<K, V>);

impl<K: Clone, V: Clone> IndexMapItems<K, V> {
    fn items(&self, range: Range<usize>) -> Vec<(K, V)> {
        let mut items = Vec::new();
        for i in range {
            if let Some((k, v)) = self.0.get_index(i) {
                items.push((k.clone(), v.clone()));
            }
        }
        items
    }
}

impl<K: Clone + 'static, V: Clone + 'static> VirtualListVector<(usize, K, V)>
    for IndexMapItems<K, V>
{
    type ItemIterator = Box<dyn Iterator<Item = (usize, K, V)>>;

    fn total_len(&self) -> usize {
        self.0.len()
    }

    fn slice(&mut self, range: Range<usize>) -> Self::ItemIterator {
        let start = range.start;
        Box::new(
            self.items(range)
                .into_iter()
                .enumerate()
                .map(move |(i, (k, v))| (i + start, k, v)),
        )
    }
}

pub fn plugin_panel(
    window_tab_data: Rc<WindowTabData>,
    position: PanelPosition,
) -> impl View {
    let config = window_tab_data.common.config;
    let plugin = window_tab_data.plugin.clone();

    stack((
        stack((
            panel_header("Installed".to_string(), config),
            installed_view(plugin.clone()),
        ))
        .style(|s| s.flex_col().width_pct(100.0).flex_grow(1.0).flex_basis(0.0)),
        stack((
            panel_header("Available".to_string(), config),
            available_view(plugin.clone()),
        ))
        .style(|s| s.flex_col().width_pct(100.0).flex_grow(1.0).flex_basis(0.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .apply_if(!position.is_bottom(), |s| s.flex_col())
    })
}

fn installed_view(plugin: PluginData) -> impl View {
    let ui_line_height = plugin.common.ui_line_height;
    let volts = plugin.installed;
    let config = plugin.common.config;
    let disabled = plugin.disabled;
    let workspace_disabled = plugin.workspace_disabled;

    let plugin_controls =
        {
            move |plugin: PluginData, volt: VoltInfo, meta: VoltMetadata| {
                let volt_id = volt.id();
                Menu::new("")
                    .entry(MenuItem::new("Reload Plugin").action({
                        let plugin = plugin.clone();
                        let meta = meta.clone();
                        move || {
                            plugin.reload_volt(meta.clone());
                        }
                    }))
                    .separator()
                    .entry(
                        MenuItem::new("Enable")
                            .enabled(disabled.with_untracked(|disabled| {
                                disabled.contains(&volt_id)
                            }))
                            .action({
                                let plugin = plugin.clone();
                                let volt = volt.clone();
                                move || {
                                    plugin.enable_volt(volt.clone());
                                }
                            }),
                    )
                    .entry(
                        MenuItem::new("Disable")
                            .enabled(disabled.with_untracked(|disabled| {
                                !disabled.contains(&volt_id)
                            }))
                            .action({
                                let plugin = plugin.clone();
                                let volt = volt.clone();
                                move || {
                                    plugin.disable_volt(volt.clone());
                                }
                            }),
                    )
                    .separator()
                    .entry(
                        MenuItem::new("Enable For Workspace")
                            .enabled(workspace_disabled.with_untracked(|disabled| {
                                disabled.contains(&volt_id)
                            }))
                            .action({
                                let plugin = plugin.clone();
                                let volt = volt.clone();
                                move || {
                                    plugin.enable_volt_for_ws(volt.clone());
                                }
                            }),
                    )
                    .entry(
                        MenuItem::new("Disable For Workspace")
                            .enabled(workspace_disabled.with_untracked(|disabled| {
                                !disabled.contains(&volt_id)
                            }))
                            .action({
                                let plugin = plugin.clone();
                                move || {
                                    plugin.disable_volt_for_ws(volt.clone());
                                }
                            }),
                    )
                    .separator()
                    .entry(MenuItem::new("Uninstall").action({
                        move || {
                            plugin.uninstall_volt(meta.clone());
                        }
                    }))
            }
        };

    let view_fn = move |volt: InstalledVoltData, plugin: PluginData| {
        let meta = volt.meta.get_untracked();
        let local_meta = meta.clone();
        let volt_id = meta.id();
        let icon = volt.icon;
        stack((
            dyn_container(
                move || icon.get(),
                move |icon| match icon {
                    None => Box::new(
                        img(move || VOLT_DEFAULT_PNG.to_vec())
                            .style(|s| s.size_full()),
                    ),
                    Some(VoltIcon::Svg(svg_str)) => Box::new(
                        svg(move || svg_str.clone()).style(|s| s.size_full()),
                    ),
                    Some(VoltIcon::Img(buf)) => {
                        Box::new(img(move || buf.clone()).style(|s| s.size_full()))
                    }
                },
            )
            .style(|s| {
                s.min_size(50.0, 50.0)
                    .size(50.0, 50.0)
                    .margin_top(5.0)
                    .margin_right(10.0)
                    .padding(5)
            }),
            stack((
                label(move || meta.display_name.clone())
                    .style(|s| s.font_bold().text_ellipsis().min_width(0.0)),
                label(move || meta.description.clone())
                    .style(|s| s.text_ellipsis().min_width(0.0)),
                stack((
                    stack((
                        label(move || meta.author.clone())
                            .style(|s| s.text_ellipsis().max_width_pct(100.0)),
                        label(move || {
                            if disabled.with(|d| d.contains(&volt_id))
                                || workspace_disabled.with(|d| d.contains(&volt_id))
                            {
                                "Disabled".to_string()
                            } else {
                                format!("v{}", meta.version.clone())
                            }
                        })
                        .style(|s| s.text_ellipsis()),
                    ))
                    .style(|s| {
                        s.justify_between()
                            .flex_grow(1.0)
                            .flex_basis(0.0)
                            .min_width(0.0)
                    }),
                    clickable_icon(
                        || LapceIcons::SETTINGS,
                        || {},
                        || false,
                        || false,
                        config,
                    )
                    .style(|s| s.padding_left(6.0))
                    .popout_menu(move || {
                        plugin_controls(
                            plugin.clone(),
                            local_meta.info(),
                            local_meta.clone(),
                        )
                    }),
                ))
                .style(|s| s.width_pct(100.0).items_center()),
            ))
            .style(|s| s.flex_col().flex_grow(1.0).flex_basis(0.0).min_width(0.0)),
        ))
        .style(|s| s.width_pct(100.0).padding_horiz(10.0).padding_vert(5.0))
    };

    container(
        scroll(
            virtual_list(
                VirtualListDirection::Vertical,
                VirtualListItemSize::Fixed(Box::new(move || {
                    ui_line_height.get() * 3.0 + 10.0
                })),
                move || IndexMapItems(volts.get()),
                move |(_, id, _)| id.clone(),
                move |(_, _, volt)| view_fn(volt, plugin.clone()),
            )
            .style(|s| s.flex_col().width_pct(100.0)),
        )
        .style(|s| s.absolute().size_pct(100.0, 100.0)),
    )
    .style(|s| {
        s.width_pct(100.0)
            .line_height(1.6)
            .flex_grow(1.0)
            .flex_basis(0.0)
    })
}

fn available_view(plugin: PluginData) -> impl View {
    let ui_line_height = plugin.common.ui_line_height;
    let volts = plugin.all.volts;
    let installed = plugin.installed;
    let config = plugin.common.config;

    let local_plugin = plugin.clone();
    let install_button = move |id: VoltID,
                               info: RwSignal<VoltInfo>,
                               installing: RwSignal<bool>| {
        let plugin = local_plugin.clone();
        let installed = create_memo(move |_| {
            installed.with(|installed| installed.contains_key(&id))
        });
        label(move || {
            if installed.get() {
                "Installed".to_string()
            } else if installing.get() {
                "Installing".to_string()
            } else {
                "Install".to_string()
            }
        })
        .disabled(move || installed.get() || installing.get())
        .style(move |s| {
            let config = config.get();
            s.color(*config.get_color(LapceColor::LAPCE_BUTTON_PRIMARY_FOREGROUND))
                .background(
                    *config.get_color(LapceColor::LAPCE_BUTTON_PRIMARY_BACKGROUND),
                )
                .margin_left(6.0)
                .padding_horiz(6.0)
                .border_radius(6.0)
        })
        .on_click(move |_| {
            plugin.install_volt(info.get_untracked());
            true
        })
        .hover_style(move |s| {
            s.cursor(CursorStyle::Pointer).background(
                config
                    .get()
                    .get_color(LapceColor::LAPCE_BUTTON_PRIMARY_BACKGROUND)
                    .with_alpha_factor(0.8),
            )
        })
        .active_style(move |s| {
            s.background(
                config
                    .get()
                    .get_color(LapceColor::LAPCE_BUTTON_PRIMARY_BACKGROUND)
                    .with_alpha_factor(0.6),
            )
        })
        .disabled_style(move |s| {
            s.background(*config.get().get_color(LapceColor::EDITOR_DIM))
        })
    };

    let view_fn = move |(_, id, volt): (usize, VoltID, AvailableVoltData)| {
        let info = volt.info.get_untracked();
        let icon = volt.icon;
        stack((
            dyn_container(
                move || icon.get(),
                move |icon| match icon {
                    None => Box::new(
                        img(move || VOLT_DEFAULT_PNG.to_vec())
                            .style(|s| s.size_full()),
                    ),
                    Some(VoltIcon::Svg(svg_str)) => Box::new(
                        svg(move || svg_str.clone()).style(|s| s.size_full()),
                    ),
                    Some(VoltIcon::Img(buf)) => {
                        Box::new(img(move || buf.clone()).style(|s| s.size_full()))
                    }
                },
            )
            .style(|s| {
                s.min_size(50.0, 50.0)
                    .size(50.0, 50.0)
                    .margin_top(5.0)
                    .margin_right(10.0)
                    .padding(5)
            }),
            stack((
                label(move || info.display_name.clone())
                    .style(|s| s.font_bold().text_ellipsis().min_width(0.0)),
                label(move || info.description.clone())
                    .style(|s| s.text_ellipsis().min_width(0.0)),
                stack((
                    label(move || info.author.clone()).style(|s| {
                        s.text_ellipsis()
                            .min_width(0.0)
                            .flex_grow(1.0)
                            .flex_basis(0.0)
                    }),
                    install_button(id, volt.info, volt.installing),
                ))
                .style(|s| s.width_pct(100.0).items_center()),
            ))
            .style(|s| s.flex_col().flex_grow(1.0).flex_basis(0.0).min_width(0.0)),
        ))
        .style(|s| s.width_pct(100.0).padding_horiz(10.0).padding_vert(5.0))
    };

    let content_rect = create_rw_signal(Rect::ZERO);

    let editor = plugin.all.query_editor.clone();
    let focus = plugin.common.focus;
    let is_focused = move || focus.get() == Focus::Panel(PanelKind::Plugin);
    let cursor_x = create_rw_signal(0.0);

    stack((
        container({
            scroll(
                text_input(editor, is_focused)
                    .on_cursor_pos(move |point| {
                        cursor_x.set(point.x);
                    })
                    .style(|s| {
                        s.padding_vert(4.0).padding_horiz(10.0).min_width_pct(100.0)
                    }),
            )
            .hide_bar(|| true)
            .on_ensure_visible(move || {
                Size::new(20.0, 0.0)
                    .to_rect()
                    .with_origin(Point::new(cursor_x.get(), 0.0))
            })
            .on_event(EventListener::PointerDown, move |_| {
                focus.set(Focus::Panel(PanelKind::Plugin));
                false
            })
            .style(move |s| {
                let config = config.get();
                s.width_pct(100.0)
                    .cursor(CursorStyle::Text)
                    .items_center()
                    .background(*config.get_color(LapceColor::EDITOR_BACKGROUND))
                    .border(1.0)
                    .border_radius(6.0)
                    .border_color(*config.get_color(LapceColor::LAPCE_BORDER))
            })
        })
        .style(|s| s.padding(10.0).width_pct(100.0)),
        container({
            scroll({
                virtual_list(
                    VirtualListDirection::Vertical,
                    VirtualListItemSize::Fixed(Box::new(move || {
                        ui_line_height.get() * 3.0 + 10.0
                    })),
                    move || IndexMapItems(volts.get()),
                    move |(_, id, _)| id.clone(),
                    view_fn,
                )
                .on_resize(move |rect| {
                    content_rect.set(rect);
                })
                .style(|s| s.flex_col().width_pct(100.0))
            })
            .on_scroll(move |rect| {
                if rect.y1 + 30.0 > content_rect.get_untracked().y1 {
                    plugin.load_more_available();
                }
            })
            .style(|s| s.absolute().size_pct(100.0, 100.0))
        })
        .style(|s| s.size_pct(100.0, 100.0)),
    ))
    .style(|s| {
        s.width_pct(100.0)
            .line_height(1.6)
            .flex_grow(1.0)
            .flex_basis(0.0)
            .flex_col()
    })
}
