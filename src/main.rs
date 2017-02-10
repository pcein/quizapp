#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;
extern crate rocket;
extern crate serde_json;
extern crate rusqlite;

#[macro_use] extern crate serde_derive;

use std::sync::Mutex;
use rocket::{State};
use rocket_contrib::Template;
use rusqlite::{Connection};

mod static_files;

type DbConn = Mutex<Connection>;

#[derive(Serialize)]
pub struct NameScore {
    team_name: String,
    total_score: i32,
}

#[derive(Serialize)]
pub struct NameQuestion {
    team_name: String,
    qn_num: i32,
}
	
#[derive(Serialize)]
struct TemplateContext {
    top_scores: Vec<NameScore>,
	recent_answers: Vec<NameQuestion>,
}

const DB_FILE: &'static str = "scores.db";

pub fn top_scores(conn: &State<DbConn>, n: usize) -> Vec<NameScore> {
    let conn = conn.lock().expect("db connection lock");
    let mut stmt = 
        conn.prepare("SELECT team_name, SUM(score)
                     FROM quiz_entry GROUP BY team_name
                     ORDER BY SUM(score) DESC"
         ).unwrap();

    let entries = stmt.query_map(&[], |row| {
            NameScore {
                team_name: row.get(0),
                total_score: row.get(1),
            }
    }).unwrap().map(|entry| entry.unwrap()).take(n).collect();

    entries
}
 
pub fn recent_answers(conn:&State<DbConn>, n: usize) -> Vec<NameQuestion> {
    let conn = conn.lock().expect("db connection lock");
    let mut stmt = 
		conn.prepare("SELECT team_name, qn_num
			      FROM quiz_entry WHERE qn_num = 
                  (SELECT MAX(qn_num) from quiz_entry)
				  ORDER BY rowid DESC"
        ).unwrap();
	
	let entries = stmt.query_map(&[], |row| {
			NameQuestion {
				team_name: row.get(0),
				qn_num: row.get(1),
			}
	}).unwrap().map(|entry| entry.unwrap()).take(n).collect();
	
	entries
}

// IF NOT EXISTS is generating a syntax error
// We will skip initialization and do it in an
// external shell script

/*fn init_database(conn:  &Connection) {

    conn.execute("CREATE TABLE if not exists quiz_entry (
                team_name TEXT NOT NULL,
                qn_num INTEGER,
                SCORE INTEGER,
                UNIQUE (team_name, qn_num)", &[])
               .expect("create table quiz_entry");
}*/


#[get("/")]
fn index(conn: State<DbConn>) -> Template {
    let context = TemplateContext {
        top_scores: top_scores(&conn, 10),
		recent_answers: recent_answers(&conn, 5),
    };
    
    Template::render("index", &context)   
}

#[post("/score/<team_name>/<qn_num>/<score>")]
fn post_score(conn: State<DbConn>, team_name: &str, qn_num: i32, score: i32) -> String {
    
	conn.lock()
    .expect("db connection lock")
    .execute("INSERT INTO quiz_entry (team_name, qn_num, score)
			 VALUES (?1, ?2, ?3)",
			 &[&team_name, &qn_num, &score]).unwrap();	
	
	format!("posted {}, {}, {}", team_name, qn_num, score)
}

fn main() {
    
    let conn = Connection::open(DB_FILE).unwrap();
     
    rocket::ignite()
        .manage(Mutex::new(conn))
        .mount("/", routes![index, post_score, static_files::all])
        .launch();
}
