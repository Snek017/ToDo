use chrono::{Local, NaiveDate, Duration};
use eframe::{egui, App, Frame};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};

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
    todos: Vec<Todo>,
    new_todo: String,
    selected_date: NaiveDate,
    scroll_offset: i32,
    current_tab: usize, // Für die Auswahl des aktuellen Tabs
}

impl TodoApp {
    fn new() -> Self {
        let todos = Self::load_from_file().unwrap_or_else(|_| Vec::new());

        TodoApp {
            todos,
            new_todo: String::new(),
            selected_date: Local::now().naive_local().date(),
            scroll_offset: 0,
            current_tab: 0, // Standardmäßig Tab 0 (Kalender)
        }
    }

    fn load_from_file() -> Result<Vec<Todo>, std::io::Error> {
        let file = File::open("todos.txt").ok();
        if let Some(file) = file {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        } else {
            Ok(Vec::new())
        }
    }

    fn save_to_file(&self) -> Result<(), std::io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("todos.txt")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.todos).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn add_todo(&mut self) {
        if !self.new_todo.is_empty() {
            let todo = Todo::new(self.new_todo.clone(), self.selected_date);
            self.todos.push(todo);
            self.save_to_file().expect("Konnte die Datei nicht speichern");
            self.new_todo.clear();
        }
    }

    fn delete_todo_at(&mut self, index: usize) {
        if index < self.todos.len() {
            self.todos.remove(index);
            self.save_to_file().expect("Konnte die Datei nicht speichern");
        }
    }

    fn get_uncompleted_tasks(&self) -> Vec<&Todo> {
        self.todos.iter().filter(|todo| !todo.completed).collect()
    }
}

impl App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.set_style(egui::Style {
            visuals: egui::Visuals {
                dark_mode: true, // Dunkles Thema aktivieren
                ..Default::default()
            },
            ..Default::default()
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Kalender").clicked() {
                    self.current_tab = 0;
                }
                if ui.button("Unerledigte Aufgaben").clicked() {
                    self.current_tab = 1;
                }
            });

            match self.current_tab {
                0 => {
                    // Kalender Tab
                    ui.heading("Kalender");

                    ui.horizontal(|ui| {
                        if ui.button("←").clicked() && self.scroll_offset > 0 {
                            self.scroll_offset -= 1;
                        }
                        ui.label("Zeitraum");
                        if ui.button("→").clicked() && self.scroll_offset < 27 {
                            self.scroll_offset += 1;
                        }
                    });

                    let today = Local::now().naive_local().date();
                    for day in self.scroll_offset..self.scroll_offset + 3 {
                        let date = today + Duration::days(day as i64);
                        let selected = date == self.selected_date;

                        if ui.selectable_label(selected, date.format("%d-%m-%Y").to_string()).clicked() {
                            self.selected_date = date;
                        }
                    }

                    ui.separator();
                    ui.heading("Aufgaben für:");
                    ui.label(self.selected_date.format("%d-%m-%Y").to_string());

                    let mut indices_to_delete = Vec::new();
                    let mut completed_changes = Vec::new();

                    for (i, todo) in self.todos.iter_mut().enumerate() {
                        if todo.date == self.selected_date {
                            ui.horizontal(|ui| {
                                let mut completed = todo.completed;
                                if ui.checkbox(&mut completed, "").clicked() {
                                    completed_changes.push((i, completed));
                                }

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

                    for (i, completed) in completed_changes {
                        self.todos[i].completed = completed;
                    }
                    self.save_to_file().expect("Konnte die Datei nicht speichern");

                    for &i in indices_to_delete.iter().rev() {
                        self.delete_todo_at(i);
                    }

                    ui.separator();
                    ui.heading("Neue Aufgabe hinzufügen");
                    ui.text_edit_singleline(&mut self.new_todo);
                    if ui.button("Hinzufügen").clicked() {
                        self.add_todo();
                    }
                }
                1 => {
                    // Unerledigte Aufgaben Tab
                    ui.heading("Unerledigte Aufgaben");

                    let uncompleted_tasks = self.get_uncompleted_tasks();

                    if uncompleted_tasks.is_empty() {
                        ui.label("Keine unerledigten Aufgaben");
                    } else {
                        for todo in uncompleted_tasks {
                            ui.horizontal(|ui| {
                                ui.label(todo.date.format("%d-%m-%Y").to_string());
                                ui.label(&todo.title);
                            });
                        }
                    }
                }
                _ => (),
            }
        });
    }
}

fn main() {
    let app = TodoApp::new();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)), // Fenstergröße
        ..Default::default()
    };

    eframe::run_native("Todo App", native_options, Box::new(|_cc| Box::new(app)));
}
