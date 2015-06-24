extern crate rustc_serialize;

use rustc_serialize::json::{Json, ToJson, ParserError, ErrorCode };
use std::str::FromStr;
use std::io::prelude::*;
use std::fs::{File, OpenOptions};

#[derive(Debug)]
struct ToDoList {
    list: Vec<String>
}

impl ToDoList {

    fn new() -> ToDoList {
        ToDoList { list: Vec::new() }
    }

    fn add(&mut self, s: String) {
        self.list.push(s);
    }

    fn remove(&mut self, id: usize) {
        if id < self.list.len() {
            let task = self.list.remove(id);
            println!("Задача #{} ({}) успешно удалена!", id, task);
        }
        
        else {
            panic!("Неверный номер элемента TODO-листа -- {}!", id);
        }
    }

    fn print(&self) {
        if self.list.len() == 0 {
            println!("Нет задач в листе!");
        }

        else {
            println!("Задачи:");
            for (i,task) in self.list.iter().enumerate() {
                println!("Задача {}: {}", i, task);
            }
        }
    }

    fn write_to_file(&self, f: &mut File) {
            f.seek(std::io::SeekFrom::Start(0)).unwrap();
            f.set_len(0).unwrap();
            f.write_all(&(self.to_json().to_string().into_bytes())).unwrap();
    }

}

impl ToJson for ToDoList {

    fn to_json(&self) -> Json {
        self.list.to_json()
    } 

}

impl FromStr for ToDoList {

    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = ParserError::SyntaxError(ErrorCode::NotUtf8,0,0);
        if let Ok(object) = s.parse::<Json>() {
            if object.is_array() {
                let mut array: Vec<String> = Vec::new();
                for e in object.as_array().unwrap().iter() {
                    array.push(e.as_string().unwrap().into());
                }
                Ok(ToDoList { list: array })
            }
            else {
                Err(error)
            }
        }
        else {
            Err(error)
        }
    }
}

fn main() {
    use std::env;
    let mut vars = env::vars();
    let args = env::args();
    let mut storage: ToDoList;
    let mut storage_file: File;
    if let Some((_,home)) = vars.find(|&(ref name, _)| name == "HOME") {
        let path = home.clone() + "/.todo";
        if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(path.clone()) {
            let mut string = String::new();
            if let Ok(_) = file.read_to_string(&mut string) {
                if let Ok(s) = string.parse::<ToDoList>() {
                    storage = s;
                }
                else {
                    panic!("Parse storage file error!");
                }
            }
            else {
                panic!("Cannot read from file to string!");
            }
            storage_file = file;
        }
        else if let Ok(mut file) = OpenOptions::new().write(true).create(true).open(path.clone()) {
            storage = ToDoList::new();
            file.write_all(storage.to_json().to_string().as_bytes()).unwrap();
            storage_file = file;
        }
        else {
            panic!("Cannot open or create storage by {}", path);
        }
    }
    else {
        panic!("Cannot find $HOME in your environment!");
    }
    if let Some(_) = env::args().position(|w| w == "list") {
        storage.print();
    }
    else if let Some(add) = env::args().position(|w| w == "add") {
        if add < (args.len() - 1) {
            storage.add(args.skip(add + 1).fold(String::new(), |s,e| s + " " + &e ));
            storage.write_to_file(&mut storage_file);
        }
        else {
            panic!("Нет описания после ключевого слова 'add'!");
        }
    }
    else if let Some(remove) = env::args().position(|w| w == "remove") {
        if remove < (args.len() - 1) {
            args.skip(remove + 1).fold((),|_, e| {
                if let Ok(id) = e.parse::<usize>() {
                    storage.remove(id);
                }
                else {
                    panic!("Невозможно удалить элемент {} -- неверный ID", e);
                }
            });
            storage.write_to_file(&mut storage_file);
        }
    }
    else {
        panic!("Нет аргументов!");
    }
}
