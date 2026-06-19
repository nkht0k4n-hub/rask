mod args;
mod utils;

use anyhow::{Context, Result};
use args::*;
use clap::Parser;
use rask::project::*;
use rask::task::*;
use rask::user::*;
use rask::Rask;
use crate::utils::user_name_to_id;

fn main() -> Result<()> {
    let args = Args::parse();

    Rask::init(args.url, args.api_key);

    match args.target {
        Target::Task(action) => match action {
            TaskAction::Create(args) => {
                let new_task = TaskRequest::new(
                    args.title,
                    args.state,
                    args.assigner_name,
                    args.project_name,
                    args.due_at,
                    args.description,
                )
                .context("Failed to create new task")?;
                Task::save(new_task).context("Failed to save new task")?;
                println!("Success to add new task");
            }
            TaskAction::List => {
                let tasks = Task::list().context("Failed to get Task list")?;

                if args.target_user!=None{
                    let users = User::list().context("Failed to get User list")?;
                    let user_id = match user_name_to_id(users, &args.target_user.unwrap()) {
                        Some(id) => id,
                        None => {
                            eprintln!("User not found");
                            return Ok(());
                        }
                    };
                    
                    let mut list_up:Vec<TaskResponse>=vec![];
                    for task in tasks {
                        if  task.assigner.id==user_id{
                            list_up.push(task);
                        }
                    }
                    let json_str = serde_json::to_string(&list_up)?;
                    println!("{}",json_str);
                }else {
                    println!("{:?}",tasks);
                }
 
            }
        },
        Target::User(action) => match action {
            UserAction::List => {
                let users = User::list().context("Failed to get User list")?;
                for user in users {
                    println!("{:?}", user);
                }
            }
        },
        Target::Project(action) => match action {
            ProjectAction::List => {
                let projects = Project::list().context("Failed to get Project list")?;
                for project in projects {
                    println!("{:?}", project);
                }
            }
        },
    }

    Ok(())
}
