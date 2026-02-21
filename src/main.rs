use editra::{self, editor::Editor};

fn main() {
    let mut editor = Editor::default();
    if let Err(e) = editor.run() {
        println!("Error: {}", e);
    }
}
