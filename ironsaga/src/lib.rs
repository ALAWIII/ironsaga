//! # ironsaga
//!
//! A Rust crate for building command pipelines with automatic compensation (rollback) support — for both sync and async workflows.
//!
//! You define plain functions, `#[ironcmd]` turns them into commands, and `IronSagaSync` / `IronSagaAsync` orchestrates their execution and rollback.
//!
//!
//! ## Why ironsaga?
//!
//! When you have a sequence of operations that must either **all succeed or all undo themselves**, manually wiring rollback logic is tedious and error-prone. ironsaga gives you:
//!
//! - ✅ Declarative command definition via `#[ironcmd]`
//! - ✅ Automatic LIFO rollback on failure
//! - ✅ Recursive compensation chains
//! - ✅ Shared typed context across commands
//!
//!
//! check out [examples](https://github.com/ALAWIII/ironsaga/tree/main/examples/src) folder for sync/async full examples.
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! ironsaga = "0.2"
//! ```
//!
//!
//! ## Defining Commands
//!
//! ```rust
//! use ironsaga::ironcmd;
//!
//! #[ironcmd]
//! pub fn greet(fname: String, lname: String) -> String {
//!     format!("Hello {} {}!", fname, lname)
//! }
//!
//! let mut cmd = Greet::new("John".into(), "Doe".into());
//! cmd.execute().unwrap();
//! assert_eq!(cmd.result(), Some("Hello John Doe!".into()));
//! ```
//!
//! The macro generates a `Greet` struct with `fname`, `lname` fields, implementing `SyncCommand`.
//!
//!
//! ## Macro Attributes
//!
//! | Attribute            | Effect                                                            |
//! |----------------------|-------------------------------------------------------------------|
//! | `result`             | if your function returns a result (failable) then its better to annotate it , this helps the macro to extend the execute trait implemetation so that it can run rollback             |
//! | `rename = "CustomStructName"`    | Override the default PascalCase generated struct name             |
//! | `recursive_rollback` | On rollback failure, recursively tries `rollback_cmd.rollback()` |
//!
//!
//! ## Shared Context
//!
//! Pass an `Rc<RefCell<YourContext>>` for sync/ Arc<Mutex<YourContext>> for sync, to share state and collect results across commands:
//!
//! ```rust
//! #[derive(Default)]
//! pub struct OrderContext {
//!     pub order_id: Option<u64>,
//!     pub rollback_log: Vec<String>,
//! }
//!
//! #[ironcmd(result, rename = "CreateOrder")]
//! pub fn create_order(id: u64, ctx: Rc<RefCell<OrderContext>>) -> anyhow::Result<u64> {
//!     ctx.borrow_mut().order_id = Some(id);
//!     Ok(id)
//! }
//! ```
//!
//!
//! ## Rollback & Compensation
//!
//! Rollbacks are also commands — inject them with `set_rollback`:
//!
//! ```rust
//! let mut create = CreateOrder::new(1001, ctx.clone());
//! create.set_rollback(CancelOrder::new(1001, ctx.clone()));
//! ```
//!
//! If the pipeline fails at step N, all previous commands roll back in **reverse order**. If a rollback itself fails and `recursive_rollback` is set, it tries `rollback_cmd.rollback()` recursively until the chain is exhausted.
//!
//!
//! ## Sync Pipeline
//!
//! ```rust
//! let mut saga = IronSagaSync::default();
//! saga.add_command(create);
//! saga.add_command(charge);
//! saga.add_command(ship); // 💥 fails → charge and create roll back
//!
//! assert!(saga.execute_all().is_err());
//! assert_eq!(ctx.borrow().rollback_log, "payment refunded");
//! assert_eq!(ctx.borrow().rollback_log, "order cancelled");[1]
//! ```
//!
//!
//! ## Async Pipeline
//!
//! `IronSagaAsync` accepts only async commands, because of design limitation to make it Send:
//!
//! ```rust
//! let mut bus = IronSagaAsync::default();
//!
//! let mut user_insertion = InsertUser::new(fname, lname, ctx.clone());
//! user_insertion.set_rollback(RemoveUserDb::new(ctx.clone()));
//!
//! bus.add_command(user_insertion);        // async
//! bus.add_command(AddBonusSalary::new(…)); // async
//! bus.add_command(AddUserRedis::new(ctx.clone()));
//!
//! assert!(bus.execute_all().await.is_err());
//! assert!(ctx.lock().unwrap().removed_user); // rollback ran ✅
//! ```
//!
//! ## Full Example
//!```no_run
#![doc=include_str!("../crate_io_example/sync_example.rs")]
//! ```

mod core;
pub use core::*;
