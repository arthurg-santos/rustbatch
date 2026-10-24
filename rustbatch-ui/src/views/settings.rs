use gtk4 as gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use rustbatch_core::model::*;
use crate::state::AppState;

pub fn build_settings_panel(
    state: Rc<RefCell<AppState>>,
) -> gtk::Widget {
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);

    let stack_switcher = gtk::StackSwitcher::builder()
        .stack(&stack)
        .build();

    let resize_page = build_resize_page(state.clone());
    stack.add_titled(&resize_page, Some("resize"), "Redimensionar");

    let format_page = build_format_page(state.clone());
    stack.add_titled(&format_page, Some("format"), "Formato");

    let filters_page = build_filters_page(state.clone());
    stack.add_titled(&filters_page, Some("filters"), "Filtros");

    let wm_page = build_watermark_page(state.clone());
    stack.add_titled(&wm_page, Some("watermark"), "Marca d'água");

    let rename_page = build_rename_page(state.clone());
    stack.add_titled(&rename_page, Some("rename"), "Renomear");

    let output_page = build_output_page(state.clone());
    stack.add_titled(&output_page, Some("output"), "Saída");

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    
    vbox.append(&stack_switcher);
    vbox.append(&stack);
    vbox.upcast()
}

fn build_resize_page(state: Rc<RefCell<AppState>>) -> gtk::Widget {
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let enable = gtk::CheckButton::with_label("Ativar redimensionamento");
    grid.attach(&enable, 0, 0, 2, 1);

    let mode_dropdown = gtk::DropDown::from_strings(&[
        "Dimensões exatas",
        "Porcentagem",
        "Caber dentro de",
    ]);
    grid.attach(&gtk::Label::new(Some("Modo:")), 0, 1, 1, 1);
    grid.attach(&mode_dropdown, 1, 1, 1, 1);

    let width_spin = gtk::SpinButton::with_range(1.0, 20_000.0, 1.0);
    width_spin.set_value(1920.0);
    grid.attach(&gtk::Label::new(Some("Largura:")), 0, 2, 1, 1);
    grid.attach(&width_spin, 1, 2, 1, 1);

    let height_spin = gtk::SpinButton::with_range(1.0, 20_000.0, 1.0);
    height_spin.set_value(1080.0);
    grid.attach(&gtk::Label::new(Some("Altura:")), 0, 3, 1, 1);
    grid.attach(&height_spin, 1, 3, 1, 1);

    let preserve = gtk::CheckButton::with_label("Preservar proporção");
    preserve.set_active(true);
    grid.attach(&preserve, 0, 4, 2, 1);

    let no_upscale = gtk::CheckButton::with_label("Não ampliar imagens pequenas");
    no_upscale.set_active(true);
    grid.attach(&no_upscale, 0, 5, 2, 1);

    enable.connect_toggled({
        let state = state.clone();
        let width_spin = width_spin.clone();
        let height_spin = height_spin.clone();
        let preserve = preserve.clone();
        let no_upscale = no_upscale.clone();
        let mode_dropdown = mode_dropdown.clone();
        move |btn| {
            let mut st = state.borrow_mut();
            if btn.is_active() {
                let mode = match mode_dropdown.selected() {
                    0 => ResizeMode::Exact {
                        width: std::num::NonZeroU32::new(width_spin.value() as u32).unwrap_or(std::num::NonZeroU32::MIN),
                        height: std::num::NonZeroU32::new(height_spin.value() as u32).unwrap_or(std::num::NonZeroU32::MIN),
                    },
                    1 => ResizeMode::Percentage(width_spin.value() as f32),
                    _ => ResizeMode::FitWithin {
                        width: std::num::NonZeroU32::new(width_spin.value() as u32).unwrap_or(std::num::NonZeroU32::MIN),
                        height: std::num::NonZeroU32::new(height_spin.value() as u32).unwrap_or(std::num::NonZeroU32::MIN),
                    },
                };
                st.config.resize = Some(ResizeConfig {
                    mode,
                    preserve_aspect: preserve.is_active(),
                    no_upscale: no_upscale.is_active(),
                });
            } else {
                st.config.resize = None;
            }
        }
    });

    grid.upcast()
}

fn build_format_page(state: Rc<RefCell<AppState>>) -> gtk::Widget {
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let format_dropdown = gtk::DropDown::from_strings(&[
        "JPEG", "PNG", "WebP (lossless)", "AVIF", "TIFF", "BMP", "GIF", "HEIC",
]);
    grid.attach(&gtk::Label::new(Some("Formato de saída:")), 0, 0, 1, 1);
    grid.attach(&format_dropdown, 1, 0, 1, 1);

    let quality_label = gtk::Label::new(Some("Qualidade:"));
    let quality_spin = gtk::SpinButton::with_range(1.0, 100.0, 1.0);
    quality_spin.set_value(85.0);
    grid.attach(&quality_label, 0, 1, 1, 1);
    grid.attach(&quality_spin, 1, 1, 1, 1);

    let png_label = gtk::Label::new(Some("Compressão PNG (0–9):"));
    let png_spin = gtk::SpinButton::with_range(0.0, 9.0, 1.0);
    png_spin.set_value(6.0);
    grid.attach(&png_label, 0, 2, 1, 1);
    grid.attach(&png_spin, 1, 2, 1, 1);

    format_dropdown.connect_selected_notify({
        let quality_label = quality_label.clone();
        let quality_spin = quality_spin.clone();
        let png_label = png_label.clone();
        let png_spin = png_spin.clone();
        let state = state.clone();
        move |dd| {
            let sel = dd.selected();
            let is_jpeg = sel == 0;
            let is_png = sel == 1;
            quality_label.set_visible(is_jpeg);
            quality_spin.set_visible(is_jpeg);
            png_label.set_visible(is_png);
            png_spin.set_visible(is_png);

            let mut st = state.borrow_mut();
            st.config.output_format = match sel {
                0 => OutputFormat::Jpeg,
                1 => OutputFormat::Png,
                2 => OutputFormat::WebP { lossless: true },
                3 => OutputFormat::Avif,
                4 => OutputFormat::Tiff,
                5 => OutputFormat::Bmp,
                6 => OutputFormat::Gif,
                7 => OutputFormat::Heic,
                _ => OutputFormat::Jpeg,
            };
            st.config.compression.quality = if is_jpeg {
                Some(quality_spin.value() as u8)
            } else {
                None
            };
            st.config.compression.png_compression = if is_png {
                Some(png_spin.value() as u8)
            } else {
                None
            };
        }
    });

    grid.upcast()
}

fn build_filters_page(state: Rc<RefCell<AppState>>) -> gtk::Widget {
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);

    let filters: &[(&str, Filter)] = &[
        ("Escala de cinza", Filter::Grayscale),
        ("Sépia", Filter::Sepia),
        ("Inverter cores", Filter::Invert),
        ("Nitidez", Filter::Sharpen),
    ];

    for (label, filter) in filters {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_top(4);
        row.set_margin_bottom(4);
        row.set_margin_start(4);
        row.set_margin_end(4);
        
        let check = gtk::CheckButton::with_label(label);
        row.append(&check);
        list.append(&row);

        check.connect_toggled({
            let state = state.clone();
            let f = *filter;
            move |btn| {
                let mut st = state.borrow_mut();
                if btn.is_active() {
                    st.config.filters.push(f);
                } else {
                    st.config.filters.retain(|x| *x != f);
                }
            }
        });
    }

    // Correção: Tipo explícito para a função de criação
    let filter_ctors: [(&str, fn(f64) -> Filter); 3] = [
        ("Brilho", |v| Filter::Brightness((v * 100.0) as i32)),
        ("Contraste", |v| Filter::Contrast(v as f32)),
        ("Saturação", |v| Filter::Saturation(v as f32)),
    ];

    for (label_text, filter_ctor) in filter_ctors {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_top(4);
        row.set_margin_bottom(4);
        row.set_margin_start(4);
        row.set_margin_end(4);
        
        let label = gtk::Label::new(Some(label_text));
        let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 3.0, 0.1);
        scale.set_value(1.0);
        row.append(&label);
        row.append(&scale);
        list.append(&row);

        scale.connect_value_changed({
            let state = state.clone();
            let ctor = filter_ctor;
            let text = label_text.to_string();
            move |sc| {
                let mut st = state.borrow_mut();
                st.config.filters.retain(|f| {
                    !matches!(
                        (f, text.as_str()),
                        (Filter::Brightness(_), "Brilho")
                            | (Filter::Contrast(_), "Contraste")
                            | (Filter::Saturation(_), "Saturação")
                    )
                });
                st.config.filters.push(ctor(sc.value()));
            }
        });
    }

    let scrolled = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();
    scrolled.upcast()
}

fn build_watermark_page(state: Rc<RefCell<AppState>>) -> gtk::Widget {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let enable = gtk::CheckButton::with_label("Ativar marca d'água");
    vbox.append(&enable);

    let text_entry = gtk::Entry::new();
    text_entry.set_placeholder_text(Some("Texto da marca d'água"));
    vbox.append(&text_entry);

    let position_dropdown = gtk::DropDown::from_strings(&[
        "Superior esquerdo", "Superior centro", "Superior direito",
        "Meio esquerdo", "Centro", "Meio direito",
        "Inferior esquerdo", "Inferior centro", "Inferior direito",
    ]);
    vbox.append(&gtk::Label::new(Some("Posição:")));
    vbox.append(&position_dropdown);

    let opacity_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.05);
    opacity_scale.set_value(0.7);
    vbox.append(&gtk::Label::new(Some("Opacidade:")));
    vbox.append(&opacity_scale);

    enable.connect_toggled({
        let state = state.clone();
        let text_entry = text_entry.clone();
        let position_dropdown = position_dropdown.clone();
        let opacity_scale = opacity_scale.clone();
        move |btn| {
            let mut st = state.borrow_mut();
            if btn.is_active() && !text_entry.text().is_empty() {
                let pos = match position_dropdown.selected() {
                    0 => WatermarkPosition::TopLeft,
                    1 => WatermarkPosition::TopCenter,
                    2 => WatermarkPosition::TopRight,
                    3 => WatermarkPosition::MiddleLeft,
                    4 => WatermarkPosition::Center,
                    5 => WatermarkPosition::MiddleRight,
                    6 => WatermarkPosition::BottomLeft,
                    7 => WatermarkPosition::BottomCenter,
                    _ => WatermarkPosition::BottomRight,
                };
                st.config.watermark = Some(Watermark::Text {
                    text: text_entry.text().to_string(),
                    font_size: 32.0,
                    color: [255, 255, 255, 255],
                    opacity: opacity_scale.value() as f32,
                    margin: 20,
                    position: pos,
                });
            } else {
                st.config.watermark = None;
            }
        }
    });

    vbox.upcast()
}

fn build_rename_page(state: Rc<RefCell<AppState>>) -> gtk::Widget {
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let prefix = gtk::Entry::new();
    prefix.set_placeholder_text(Some("Prefixo"));
    grid.attach(&gtk::Label::new(Some("Prefixo:")), 0, 0, 1, 1);
    grid.attach(&prefix, 1, 0, 1, 1);

    let suffix = gtk::Entry::new();
    suffix.set_placeholder_text(Some("Sufixo"));
    grid.attach(&gtk::Label::new(Some("Sufixo:")), 0, 1, 1, 1);
    grid.attach(&suffix, 1, 1, 1, 1);

    let replace_from = gtk::Entry::new();
    replace_from.set_placeholder_text(Some("Substituir"));
    grid.attach(&gtk::Label::new(Some("De:")), 0, 2, 1, 1);
    grid.attach(&replace_from, 1, 2, 1, 1);

    let replace_to = gtk::Entry::new();
    replace_to.set_placeholder_text(Some("Por"));
    grid.attach(&gtk::Label::new(Some("Para:")), 0, 3, 1, 1);
    grid.attach(&replace_to, 1, 3, 1, 1);

    let start_spin = gtk::SpinButton::with_range(0.0, 999_999.0, 1.0);
    start_spin.set_value(1.0);
    grid.attach(&gtk::Label::new(Some("Número inicial:")), 0, 4, 1, 1);
    grid.attach(&start_spin, 1, 4, 1, 1);

    let padding_spin = gtk::SpinButton::with_range(0.0, 10.0, 1.0);
    padding_spin.set_value(4.0);
    grid.attach(&gtk::Label::new(Some("Zeros à esquerda:")), 0, 5, 1, 1);
    grid.attach(&padding_spin, 1, 5, 1, 1);

    let include_date = gtk::CheckButton::with_label("Incluir data");
    grid.attach(&include_date, 0, 6, 2, 1);

    for entry in [&prefix, &suffix, &replace_from, &replace_to] {
        entry.connect_changed({
            let state = state.clone();
            let prefix = prefix.clone();
            let suffix = suffix.clone();
            let replace_from = replace_from.clone();
            let replace_to = replace_to.clone();
            let start_spin = start_spin.clone();
            let padding_spin = padding_spin.clone();
            let include_date = include_date.clone();
            move |_| {
                let mut st = state.borrow_mut();
                st.config.rename = RenameConfig {
                    prefix: prefix.text().to_string(),
                    suffix: suffix.text().to_string(),
                    replace_from: replace_from.text().to_string(),
                    replace_to: replace_to.text().to_string(),
                    start_index: start_spin.value() as u32,
                    index_padding: padding_spin.value() as u8,
                    include_date: include_date.is_active(),
                };
            }
        });
    }

    grid.upcast()
}

fn build_output_page(state: Rc<RefCell<AppState>>) -> gtk::Widget {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let choose_btn = gtk::Button::with_label("Escolher pasta de destino");
    let path_label = gtk::Label::new(Some("Nenhuma pasta selecionada"));
    path_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);

    choose_btn.connect_clicked({
        let state = state.clone();
        let path_label = path_label.clone();
        move |btn| {
            let dialog = gtk::FileDialog::builder()
                .title("Selecionar pasta de destino")
                .modal(true)
                .build();
            let parent = btn.root().and_downcast::<gtk::Window>();
            dialog.select_folder(
                parent.as_ref(),
                gtk::gio::Cancellable::NONE,
                {
                    let state = state.clone();
                    let path_label = path_label.clone();
                    move |result| {
                        // CORREÇÃO: select_folder retorna Result<File, Error> (sem Option)
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                path_label.set_text(&path.display().to_string());
                                let mut st = state.borrow_mut();
                                st.config.output.directory = path;
                            }
                        }
                    }
                },
            );
        }
    });

    vbox.append(&choose_btn);
    vbox.append(&path_label);

    let collision_dropdown = gtk::DropDown::from_strings(&[
        "Nome exclusivo", "Substituir", "Ignorar existente",
    ]);
    vbox.append(&gtk::Label::new(Some("Se o arquivo existir:")));
    vbox.append(&collision_dropdown);

    collision_dropdown.connect_selected_notify({
        let state = state.clone();
        move |dd| {
            let mut st = state.borrow_mut();
            st.config.output.collision = match dd.selected() {
                0 => CollisionPolicy::Unique,
                1 => CollisionPolicy::Overwrite,
                _ => CollisionPolicy::Skip,
            };
        }
    });

    let subfolder = gtk::CheckButton::with_label("Criar subpasta de saída");
    vbox.append(&subfolder);
    subfolder.connect_toggled({
        let state = state.clone();
        move |btn| {
            let mut st = state.borrow_mut();
            st.config.output.create_subfolder = btn.is_active();
        }
    });

    vbox.upcast()
}