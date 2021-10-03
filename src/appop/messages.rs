use crate::appop::{AppOp, EpubBook};


impl AppOp {
    pub fn open_page_message(&self) -> glib::Sender<usize> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        
        rx.attach(None, glib::clone!(@strong self.app_runtime as app_runtime => @default-return glib::Continue(false), move |data| {
            app_runtime.update_state_with(move |state| {
                state.open_page(data);
            });
            
            glib::Continue(true)
        }));
        
        tx
    }
    
    pub fn open_page_send(&self, num: usize) {
        let _ = self
            .open_page_sender
            .as_ref()
            .unwrap()
            .send(num);
    }
    
    pub fn open_epub_book_message(&self) -> glib::Sender<EpubBook> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        
        rx.attach(None, glib::clone!(@strong self.app_runtime as app_runtime => @default-return glib:Continue(false), move |data| {
            app_runtime.update_state_with(move |state| {
                state.post_open_epub_book(data);
            });
            
            glib::Continue(true)
        }));
        
        tx
    }
    
    // pub fn open_epub_book_send(&self, book: EpubBook) {
    //     self
    //         .open_epub_book_sender
    //         .as_ref()
    //         .unwrap()
    //         .send(book)
    //         .expect("Cannot send PathBuf message");
    // }
}