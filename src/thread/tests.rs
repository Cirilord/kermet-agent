use super::{Thread, ThreadStore, ThreadStoreError, user_directory};
use std::{
  fs,
  path::{Path, PathBuf},
};

fn temporary_directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join(format!("kermet-{name}-{}", rand::random::<u64>()));
  fs::create_dir(&directory).expect("the temporary directory should be created");
  return directory;
}

#[test]
fn saves_and_loads_a_thread() -> () {
  let directory = temporary_directory("thread-store");
  let store = ThreadStore::new(directory.clone());
  let mut thread = Thread::new(
    "thread-0123456789abcdef".to_owned(),
    PathBuf::from("/workspace"),
    "default".to_owned(),
  );

  thread.record_user_message("Inspect the project".to_owned());
  thread.record_assistant_message("The project is a Rust CLI.".to_owned());

  store.save(&thread).expect("the thread should be saved");
  let loaded = store
    .load(thread.id())
    .expect("the thread should be loaded");

  fs::remove_dir_all(directory).expect("the temporary directory should be removed");

  assert_eq!(loaded.workspace(), Path::new("/workspace"));
  assert_eq!(loaded.profile(), "default");
  assert!(
    loaded
      .prompt("What is next?")
      .contains("Inspect the project")
  );
  assert_eq!(
    loaded.render_transcript(),
    "> Inspect the project\nThe project is a Rust CLI.\n"
  );
  return;
}

#[test]
fn lists_saved_threads_by_identifier() -> () {
  let directory = temporary_directory("thread-list");
  let store = ThreadStore::new(directory.clone());
  let first = Thread::new(
    "thread-aaaaaaaaaaaaaaaa".to_owned(),
    PathBuf::from("/workspace"),
    "default".to_owned(),
  );
  let second = Thread::new(
    "thread-bbbbbbbbbbbbbbbb".to_owned(),
    PathBuf::from("/workspace"),
    "default".to_owned(),
  );

  store
    .save(&second)
    .expect("the second thread should be saved");
  store
    .save(&first)
    .expect("the first thread should be saved");
  let threads = store.list().expect("the threads should be listed");

  fs::remove_dir_all(directory).expect("the temporary directory should be removed");

  assert_eq!(threads.len(), 2);
  assert_eq!(threads[0].id(), "thread-aaaaaaaaaaaaaaaa");
  assert_eq!(threads[1].id(), "thread-bbbbbbbbbbbbbbbb");

  return;
}

#[test]
fn rejects_an_invalid_thread_identifier() -> () {
  let directory = temporary_directory("thread-id");
  let store = ThreadStore::new(directory.clone());

  let error = store
    .load("../thread")
    .expect_err("the identifier should be rejected");

  fs::remove_dir_all(directory).expect("the temporary directory should be removed");

  assert_eq!(error, ThreadStoreError::InvalidId);
  return;
}

#[test]
fn derives_a_thread_directory_from_the_user_home() -> () {
  assert_eq!(
    user_directory(Path::new("/home/user")),
    PathBuf::from("/home/user/.kermet/threads")
  );
  return;
}
