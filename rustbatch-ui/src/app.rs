use gtk4 as gtk;
use gtk::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use crate::state::{AppState, UiJob};
use crate::bridge;
use crate::views::settings::build_settings_panel;
use crate::views::preview::PreviewWidget;

pub fn build_ui(app: &gtk::Application) {
    let state = Rc::new(RefCell::new(AppState::default()));
    let window = gtk::ApplicationWindow::builder()
        .application(app).title("RustBatch").default_width(1100).default_height(720).build();

    let header = gtk::HeaderBar::new();
    let import_btn = gtk::Button::with_label("Importar imagens");
    let clear_btn = gtk::Button::with_label("Limpar fila");
    header.pack_start(&import_btn);
    header.pack_end(&clear_btn);
    window.set_titlebar(Some(&header));

    let list_store = gio::ListStore::new::<glib::BoxedAnyObject>();
    let selection = gtk::SingleSelection::new(Some(list_store.clone()));
    let column_view = gtk::ColumnView::new(Some(selection.clone()));
    
    let factory_name = gtk::SignalListItemFactory::new();
    factory_name.connect_setup(|_, item| {
        item.downcast_ref::<gtk::ListItem>().expect("ListItem").set_child(Some(&gtk::Label::new(None)));
    });
    factory_name.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().expect("ListItem");
        let obj = item.item().and_downcast::<glib::BoxedAnyObject>().expect("Boxed");
        let job: std::cell::Ref<UiJob> = obj.borrow();
        let label = item.child().and_downcast::<gtk::Label>().expect("Label");
        label.set_text(&format!("{} ({}x{})", job.name, job.width, job.height));
    });
    column_view.append_column(&gtk::ColumnViewColumn::new(Some("Arquivo"), Some(factory_name)));

    let scrolled = gtk::ScrolledWindow::builder().child(&column_view).vexpand(true).build();

    let preview = PreviewWidget::new();
    let settings_panel = build_settings_panel(state.clone());
    
    let right_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    right_box.set_margin_top(8);
    right_box.set_margin_bottom(8);
    right_box.set_margin_start(8);
    right_box.set_margin_end(8);
    
    right_box.append(&preview.widget);
    right_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    right_box.append(&settings_panel);

    // NOVO: Envolve o painel direito em um ScrolledWindow para ele não forçar a janela a crescer
    let right_scrolled = gtk::ScrolledWindow::builder()
        .child(&right_box)
        .vexpand(true)
        .build();

    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_start_child(Some(&scrolled));
    paned.set_end_child(Some(&right_scrolled)); // <-- Usa a nova variável
    paned.set_position(600);
    paned.set_vexpand(true);
    
    // NOVO: Permite que os painéis encolham, para a barra inferior caber na tela
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(true);
    paned.set_shrink_end_child(true);

    let progress = gtk::ProgressBar::new();
    progress.set_show_text(true);
    progress.set_hexpand(true); 
    
    let start_btn = gtk::Button::with_label("Iniciar processamento");
    let cancel_btn = gtk::Button::with_label("Cancelar");
    cancel_btn.set_sensitive(false);

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    bottom.set_margin_top(8);
    bottom.set_margin_bottom(8);
    bottom.set_margin_start(8);
    bottom.set_margin_end(8);
    
    bottom.append(&progress);
    bottom.append(&start_btn);
    bottom.append(&cancel_btn);
    
    // NOVO: Garante que a barra inferior mantenha sua altura natural
    bottom.set_vexpand(false);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&paned);
    root.append(&bottom);
    window.set_child(Some(&root));

    let preview_clone = preview.clone();
    let state_clone = state.clone();
    selection.connect_selected_notify(move |sel| {
        let idx = sel.selected();
        if idx != gtk::INVALID_LIST_POSITION {
            if let Some(obj) = sel.item(idx).and_downcast::<glib::BoxedAnyObject>() {
                let job: std::cell::Ref<UiJob> = obj.borrow();
                let cfg = state_clone.borrow().config.clone();
                preview_clone.update(Some(&job.path), &cfg);
            }
        } else {
            preview_clone.update(None, &state_clone.borrow().config);
        }
    });

    // Corrigido: Removida a linha duplicada que estava aqui
    let drop_target = gtk::DropTarget::new(gtk::gdk::FileList::static_type(), gtk::gdk::DragAction::COPY);
    drop_target.connect_drop({
        let list_store = list_store.clone();
        let state = state.clone();
        move |_, value, _, _| {
            if let Ok(file_list) = value.get::<gtk::gdk::FileList>() {
                for file in file_list.files() {
                    if let Some(path) = file.path() { 
                        add_job(&list_store, &state, path); 
                    }
                }
            }
            true
        }
    });
    window.add_controller(drop_target);

    import_btn.connect_clicked({
        let window = window.clone(); let list_store = list_store.clone(); let state = state.clone();
        move |_| {
            let dialog = gtk::FileDialog::builder().title("Selecionar imagens").modal(true).build();
            dialog.open_multiple(Some(&window), gtk::gio::Cancellable::NONE, {
                let list_store = list_store.clone(); let state = state.clone();
                move |result| {
                    if let Ok(files) = result {
                        for i in 0..files.n_items() {
                            if let Some(file) = files.item(i).and_downcast::<gtk::gio::File>() {
                                if let Some(path) = file.path() { 
                                    add_job(&list_store, &state, path); 
                                }
                            }
                        }
                    }
                }
            });
        }
    });

    start_btn.connect_clicked({
        let list_store = list_store.clone(); let state = state.clone();
        let progress = progress.clone(); let cancel_btn = cancel_btn.clone();
        move |btn| {
            let st = state.borrow();
            if st.jobs.is_empty() { return; }
            btn.set_sensitive(false); cancel_btn.set_sensitive(true);

            let jobs: Vec<_> = st.jobs.iter().map(|j| j.to_image_job()).collect();
            let cfg = st.config.clone(); let cancel = st.cancel.clone();
            cancel.store(false, std::sync::atomic::Ordering::Relaxed);

            let (tx, rx) = async_channel::unbounded();
            rustbatch_core::run_batch(jobs, cfg, cancel, tx);
            bridge::spawn_batch_listener(rx, list_store.clone(), state.clone(), progress.clone(), btn.clone(), cancel_btn.clone());
        }
    });

    cancel_btn.connect_clicked({
        let state = state.clone();
        move |_| { state.borrow().cancel.store(true, std::sync::atomic::Ordering::Relaxed); }
    });

    window.present();
}

fn add_job(list_store: &gio::ListStore, state: &Rc<RefCell<AppState>>, path: PathBuf) {
    if let Ok(meta) = rustbatch_core::discovery::inspect_file(&path) {
        let ui_job = UiJob {
            name: path.file_name().and_then(|n| n.to_str()).unwrap_or("(sem nome)").to_string(),
            width: meta.width, height: meta.height, path: path.clone(),
        };
        
        {
            let mut st = state.borrow_mut();
            st.jobs.push(ui_job.clone());
        }
        
        list_store.append(&glib::BoxedAnyObject::new(ui_job));
    }
}