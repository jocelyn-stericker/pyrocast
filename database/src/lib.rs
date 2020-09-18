use directories::ProjectDirs;
use loader::{Loader, Query as LoaderQuery};
use sqlite::{Connection, OpenFlags};
use state::{ChannelRef, CurrentState, StateAction};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

pub enum DatabaseAction {
    Subscribe(ChannelRef),
    Unsubscribe(ChannelRef),
}

fn database_thread(recv: Receiver<DatabaseAction>, current: Arc<CurrentState>, loader: Loader) {
    let dirs =
        ProjectDirs::from("ca", "nettek", "Pyrocast").expect("Cannot find project directories");
    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(&data_dir).expect("Cannot create project data directory");
    let db_name = data_dir
        .join("pyrocast.sqlite")
        .to_str()
        .expect("Data directory must be utf-8")
        .to_owned();

    eprintln!("Connecting to {}", &db_name);

    // TODO: error handling
    let connection =
        Connection::open_with_flags(db_name, OpenFlags::new().set_create().set_read_write())
            .unwrap();

    connection
        .execute(
            r#"
            create table if not exists meta(
                id      text primary key not null,
                value   text not null
            )
        "#,
        )
        .unwrap();

    let mut get_db_version = connection
        .prepare(r#"select value from meta where id = "version""#)
        .unwrap();
    let db_version = if get_db_version.next().unwrap() == sqlite::State::Row {
        get_db_version.read::<i64>(0).unwrap_or(0)
    } else {
        0
    };

    if db_version == 0 {
        connection.execute(include_str!("./schema.sql")).unwrap();
    }

    let mut subscriptions: Vec<String> = vec![];

    let mut get_subscriptions = connection
        .prepare(r#"select pk from subscription"#)
        .unwrap();

    while get_subscriptions.next().unwrap() == sqlite::State::Row {
        subscriptions.push(get_subscriptions.read::<String>(0).unwrap());
    }

    let send_subscriptions = |subscriptions: &[String]| {
        current.update(vec![StateAction::SetSubscriptions(Ok(subscriptions
            .iter()
            .map(|pk| current.get().channel_ref(pk.to_owned()))
            .collect()))]);
    };

    send_subscriptions(&subscriptions);

    for pk in &subscriptions {
        loader.queue(LoaderQuery::ItunesLookup { pk: pk.to_owned() });
    }

    let mut add_subscription = connection
        .prepare(r#"insert into subscription values(?)"#)
        .unwrap();

    let mut remove_subscription = connection
        .prepare(r#"delete from subscription where pk = ?"#)
        .unwrap();

    while let Ok(ev) = recv.recv() {
        match ev {
            DatabaseAction::Subscribe(channel) => {
                let channel_pk = channel.pk().to_owned();
                if !subscriptions.contains(&channel_pk) {
                    add_subscription.bind(1, &channel_pk as &str).unwrap();
                    add_subscription.next().unwrap();
                    add_subscription.reset().unwrap();

                    subscriptions.push(channel_pk.clone());
                    send_subscriptions(&subscriptions);
                    loader.queue(LoaderQuery::ItunesLookup { pk: channel_pk });
                }
            }
            DatabaseAction::Unsubscribe(channel) => {
                if let Some(pos) = subscriptions.iter().position(|x| x == channel.pk()) {
                    let channel_pk = channel.pk();
                    remove_subscription.bind(1, channel_pk).unwrap();
                    remove_subscription.next().unwrap();
                    remove_subscription.reset().unwrap();

                    subscriptions.remove(pos);
                    send_subscriptions(&subscriptions);
                }
            }
        }
    }
}

pub fn new_database(current: Arc<CurrentState>, loader: Loader) -> Sender<DatabaseAction> {
    let (send_cmd, recv_cmd) = channel();
    std::thread::spawn(move || {
        database_thread(recv_cmd, current, loader);
    });

    send_cmd
}
