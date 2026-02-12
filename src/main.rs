use std::env;
use std::io::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::fs::OpenOptions;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn memo_dir() -> PathBuf {
    let home = env::var("HOME").unwrap();
    let path = PathBuf::from(home).join(".memo");
    fs::create_dir_all(&path).unwrap();
    path
}

fn memo_path(name: &str) -> PathBuf {
    memo_dir().join(format!("{}.txt", name))
}

fn config_dir() -> PathBuf {
    let dir = memo_dir().join("config");
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn editor_path() -> PathBuf {
    config_dir().join("editor")
}

fn set_editor(editor: &str) {
    let path = editor_path();
    fs::write(&path, editor).unwrap();
    println!("Editor set to '{}'", editor);
}

fn get_editor() -> String {
    let path = editor_path();
    if let Ok(editor) = fs::read_to_string(path) {
        return editor.trim().to_string();
    }
    "vi".to_string()
}

fn add(name: &str, content: &str) {
    let path = memo_path(name);
    if !path.exists() {
        println!("Memo '{}' does not exist.", name);
        return;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap();

    writeln!(file, "{}", content).unwrap();
}

fn create(name: &str) {
    let path = memo_path(name);
    if path.exists() {
        println!("Memo '{}' already exists.", name);
    };
    fs::write(path, "").unwrap();
}

fn delete(name: &str) {
    let path = memo_path(name);
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

fn update(old_name: &str, new_name: &str) {
    let old_path = memo_path(old_name);
    if !old_path.exists() {
        println!("Memo '{}' does not exist.", old_name);
        return;
    }

    let new_path = memo_path(new_name);
    fs::rename(old_path, new_path).unwrap();
}

fn read(name: &str) {
    let path = memo_path(name);
    if !path.exists() {
        println!("Memo '{}' does not exist.", name);
        return;
    }

    let content = fs::read_to_string(path).unwrap();
    println!("{}\n{}", name, "~".repeat(40 - name.len()));
    println!("{}", content);
    println!("{}", "~".repeat(40));
}

fn edit(name: &str) {
    let path = memo_path(name);
    if !path.exists() {
        println!("Memo '{}' does not exist.", name);
        return;
    }

    let editor = get_editor();
    Command::new(editor)
        .arg(&path)
        .status()
        .unwrap();
}

fn list() {
    for entry in fs::read_dir(memo_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            if let Some(name) = path.file_stem() {
                println!("{}", name.to_string_lossy());
            }
        }
    }
}

fn show_editor() {
    let editor = get_editor();
    println!("Current editor: '{}'", editor);
}

fn select_memo() -> Option<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("note ls | fzf --height 40% --layout=reverse --reverse --prompt '❯ Memo> '")
        .output()
        .ok()?;

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn edit_interactive(arg: Option<&String>) {
    let name = match arg {
        Some(n) => n.to_string(),
        None => match select_memo() {
            Some(n) => n,
            None => return,
        },
    };
    edit(&name);
}

fn read_interactive(arg: Option<&String>) {
    let name = match arg {
        Some(n) => n.to_string(),
        None => match select_memo() {
            Some(n) => n,
            None => return,
        },
    };
    read(&name);
}

fn delete_interactive(arg: Option<&String>) {
    let name = match arg {
        Some(n) => n.to_string(),
        None => match select_memo() {
            Some(n) => n,
            None => return,
        },
    };
    delete(&name);
}

fn print_help() {
    println!("note - simple memo CLI\n");
    println!("Usage:");
    println!("  note <command> [args]\n");
    println!("Commands:");
    println!("  create <name>            Create a new memo");
    println!("  delete <name>            Delete a memo (fzf if no arg)");
    println!("  update <old> <new>       Rename a memo");
    println!("  read <name>              Read a memo (fzf if no arg)");
    println!("  edit <name>              Edit a memo (fzf if no arg)");
    println!("  add <name> <content...>  Append content to a memo");
    println!("  ls                        List all memos");
    println!("  set editor <cmd>          Set editor command");
    println!("  editor                    Show current editor\n");
    println!("Options:");
    println!("  -h, --help               Show this help");
    println!("  -v, --version            Show version");
}

fn print_version() {
    println!("note {}", VERSION);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "create" => create(&args[2]),
        "delete" => {
            if args.len() >= 3 {
                delete_interactive(Some(&args[2]));
            } else {
                delete_interactive(None);
            }
        }
        "update" => update(&args[2], &args[3]),
        "read" => {
            if args.len() >= 3 {
                read_interactive(Some(&args[2]));
            } else {
                read_interactive(None);
            }
        }
        "edit" => {
            if args.len() >= 3 {
                edit_interactive(Some(&args[2]));
            } else {
                edit_interactive(None);
            }
        }
        "add" => add(&args[2], &args[3]),
        "ls" => list(),
        "editor" => show_editor(),
        "set" => {
            if args.len() >= 4 && args[2] == "editor" {
                set_editor(&args[3]);
            } else {
                println!("Usage: note set editor <editor>");
            }
        }
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-v" => print_version(),
        _ => println!("unknown command"),
    }
}

