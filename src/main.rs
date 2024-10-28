use chrono::{Local, NaiveDate}; // Datelike entfernt
use eframe::{egui, epi};
use serde::{Deserialize, Serialize};
use sled::Db;
use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Todo {
    title: String,
    date: NaiveDate,
    completed: bool,
}

impl Todo {
    fn new(title: String, date: NaiveDate) -> Self {
        Todo { title, date, completed: false }
    }
}

struct TodoApp {
    db: Db,
    todos: Vec<Todo>,
    new_todo: String,
    selected_date: NaiveDate,
    scroll_offset: i32,
}

impl TodoApp {
    fn new(db: Db) -> Self {
        let todos = db
            .iter()
            .values()
            .filter_map(|res| res.ok())
            .filter_map(|ivec| deserialize::<Todo>(&ivec).ok())
            .collect();

        TodoApp {
            db,
            todos,
            new_todo: String::new(),
            selected_date: Local::now().naive_local().date(), // Umwandlung in NaiveDate
            scroll_offset: 0,
        }
    }

    fn save_to_db(&self) {
        for (i, todo) in self.todos.iter().enumerate() {
            let _ = self.db.insert(i.to_string(), serialize(todo).unwrap());
        }
    }

    fn add_todo(&mut self) {
        if !self.new_todo.is_empty() {
            let todo = Todo::new(self.new_todo.clone(), self.selected_date);
            self.todos.push(todo);
            self.save_to_db();
            self.new_todo.clear();
        }
    }

    fn delete_todo_at(&mut self, index: usize) {
        if index < self.todos.len() {
            self.todos.remove(index);
            self.db.remove(index.to_string()).ok();
            self.save_to_db();
        }
    }
}

impl epi::App for TodoApp {
    fn name(&self) -> &str {
        "TOP" // App-Name geändert
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut eframe::epi::Frame<'_>) { // `_frame` benutzt
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Kalender");

            // Navigation für 3-Tage-Anzeige mit Scrollfunktion
            ui.horizontal(|ui| {
                if ui.button("←").clicked() && self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                ui.label("Zeitraum"); // Zeitraum-Label geändert
                if ui.button("→").clicked() && self.scroll_offset < 27 {
                    self.scroll_offset += 1;
                }
            });

            let today = Local::now().naive_local().date(); // Umwandlung in NaiveDate
            for day in self.scroll_offset..self.scroll_offset + 3 {
                let date = today + chrono::Duration::days(day as i64);
                let selected = date == self.selected_date;

                if ui.selectable_label(selected, date.to_string()).clicked() {
                    self.selected_date = date;
                }
            }

            ui.separator();
            ui.heading("Aufgaben für: ");
            ui.label(self.selected_date.to_string());

            let mut indices_to_delete = Vec::new();
            let mut completed_changes = Vec::new();

            for (i, todo) in self.todos.iter_mut().enumerate() {
                if todo.date == self.selected_date {
                    ui.horizontal(|ui| {
                        let mut completed = todo.completed;
                        if ui.checkbox(&mut completed, "").clicked() {
                            completed_changes.push((i, completed));
                        }

                        // Grün färben für erledigte Aufgaben
                        let label_color = if todo.completed {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::WHITE
                        };
                        ui.colored_label(label_color, &todo.title);

                        if ui.button("Entfernen").clicked() {
                            indices_to_delete.push(i);
                        }
                    });
                }
            }

            // Anwenden der gespeicherten Änderungen nach der Schleife
            for (i, completed) in completed_changes {
                self.todos[i].completed = completed;
            }
            self.save_to_db(); // Einmalige Speicherung nach Änderungen

            for &i in indices_to_delete.iter().rev() {
                self.delete_todo_at(i);
            }

            ui.separator();
            ui.heading("Neue Aufgabe hinzufügen");
            ui.text_edit_singleline(&mut self.new_todo);
            if ui.button("Hinzufügen").clicked() {
                self.add_todo();
            }
        });
    }
}

fn main() {
    let db = sled::open("todo_db").expect("Datenbank konnte nicht geöffnet werden");
    let app = TodoApp::new(db);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
