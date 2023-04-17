mod message;

#[macro_use]
extern crate rocket;

use message::Message;
use rocket::{
    form::Form,
    fs::{relative, FileServer},
    response::stream::{Event, EventStream},
    tokio::{
        select,
        sync::broadcast::{channel, error::RecvError, Sender},
    },
    Shutdown, State,
};

#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    // the result failing here means no users are active so its ok?
    let _res = queue.send(form.into_inner());
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();

    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![post, events])
        .mount("/", FileServer::from(relative!("static")))
}
