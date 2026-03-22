<!-- crate documentation start -->
 ## ironsaga

 A Rust crate for building command pipelines with automatic compensation (rollback) support — for both sync and async workflows.

 You define plain functions, `#[ironcmd]` turns them into commands, and `IronSagaSync` / `IronSagaAsync` orchestrates their execution and rollback.


 ### Why ironsaga?

 When you have a sequence of operations that must either **all succeed or all undo themselves**, manually wiring rollback logic is tedious and error-prone. ironsaga gives you:

 - ✅ Declarative command definition via `#[ironcmd]`
 - ✅ Automatic LIFO rollback on failure
 - ✅ Recursive compensation chains
 - ✅ Shared typed context across commands
 - ✅ Mix sync and async commands freely in one pipeline


 check out [examples](https://github.com/ALAWIII/ironsaga/tree/main/examples/src) folder for sync/async full examples.
 ### Quick Start

 ```toml
 [dependencies]
 ironsaga = "0.1"
 ```


 ### Defining Commands

 ```rust
 use ironsaga::ironcmd;

 #[ironcmd]
 pub fn greet(fname: String, lname: String) -> String {
     format!("Hello {} {}!", fname, lname)
 }

 let mut cmd = Greet::new("John".into(), "Doe".into());
 cmd.execute().unwrap();
 assert_eq!(cmd.result(), Some("Hello John Doe!".into()));
 ```

 The macro generates a `Greet` struct with `fname`, `lname` fields, implementing `SyncCommand`.


 ### Macro Attributes

 | Attribute            | Effect                                                            |
 |----------------------|-------------------------------------------------------------------|
 | `result`             | if your function returns a result (failable) then its better to annotate it , this helps the macro to extend the execute trait implemetation so that it can run rollback             |
 | `rename = "CustomStructName"`    | Override the default PascalCase generated struct name             |
 | `recursive_rollback` | On rollback failure, recursively tries `rollback_cmd.rollback()` |


 ### Shared Context

 Pass an `Rc<RefCell<YourContext>>` to share state and collect results across commands:

 ```rust
 #[derive(Default)]
 pub struct OrderContext {
     pub order_id: Option<u64>,
     pub rollback_log: Vec<String>,
 }

 #[ironcmd(result, rename = "CreateOrder")]
 pub fn create_order(id: u64, ctx: Rc<RefCell<OrderContext>>) -> anyhow::Result<u64> {
     ctx.borrow_mut().order_id = Some(id);
     Ok(id)
 }
 ```


 ### Rollback & Compensation

 Rollbacks are also commands — inject them with `set_rollback`:

 ```rust
 let mut create = CreateOrder::new(1001, ctx.clone());
 create.set_rollback(CancelOrder::new(1001, ctx.clone()));
 ```

 If the pipeline fails at step N, all previous commands roll back in **reverse order**. If a rollback itself fails and `recursive_rollback` is set, it tries `rollback_cmd.rollback()` recursively until the chain is exhausted.


 ### Sync Pipeline

 ```rust
 let mut saga = IronSagaSync::default();
 saga.add_sync_command(create);
 saga.add_sync_command(charge);
 saga.add_sync_command(ship); // 💥 fails → charge and create roll back

 assert!(saga.execute_all().is_err());
 assert_eq!(ctx.borrow().rollback_log, "payment refunded");
 assert_eq!(ctx.borrow().rollback_log, "order cancelled");[1]
 ```


 ### Async Pipeline

 `IronSagaAsync` accepts both sync and async commands freely:

 ```rust
 let mut bus = IronSagaAsync::default();

 let mut user_insertion = InsertUser::new(fname, lname, ctx.clone());
 user_insertion.set_rollback_async(RemoveUserDb::new(ctx.clone()));

 bus.add_async_command(user_insertion);        // async
 bus.add_sync_command(AddBonusSalary::new(…)); // sync — mixed freely
 bus.add_async_command(AddUserRedis::new(ctx.clone()));

 assert!(bus.execute_all().await.is_err());
 assert!(ctx.borrow().removed_user); // rollback ran ✅
 ```

 ### Full Example
```rust
use ironsaga::{IronSagaSync, anyhow, ironcmd};
use std::{cell::RefCell, rc::Rc};

// ── Shared Context ────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct OrderContext {
    pub order_id: Option<u64>,
    pub payment_id: Option<u64>,
    pub shipment_id: Option<u64>,
    pub rollback_log: Vec<String>,
}

// ── Step 1: Create Order ──────────────────────────────────────────────────────

#[ironcmd(result, rename = "CreateOrder")]
pub fn create_order(order_id: u64, ctx: Rc<RefCell<OrderContext>>) -> anyhow::Result<u64> {
    println!("[1] Creating order #{order_id}...");
    ctx.borrow_mut().order_id = Some(order_id);
    Ok(order_id)
}

#[ironcmd(result, rename = "CancelOrder")]
pub fn cancel_order(order_id: u64, ctx: Rc<RefCell<OrderContext>>) -> anyhow::Result<()> {
    println!("[↩] Cancelling order #{order_id}...");
    ctx.borrow_mut()
        .rollback_log
        .push(format!("order #{order_id} cancelled"));
    Ok(())
}

// ── Step 2: Charge Payment ────────────────────────────────────────────────────

#[ironcmd(result, rename = "ChargePayment")]
pub fn charge_payment(payment_id: u64, ctx: Rc<RefCell<OrderContext>>) -> anyhow::Result<u64> {
    println!("[2] Charging payment #{payment_id}...");
    ctx.borrow_mut().payment_id = Some(payment_id);
    Ok(payment_id)
}

#[ironcmd(result, rename = "RefundPayment")]
pub fn refund_payment(payment_id: u64, ctx: Rc<RefCell<OrderContext>>) -> anyhow::Result<()> {
    println!("[↩] Refunding payment #{payment_id}...");
    ctx.borrow_mut()
        .rollback_log
        .push(format!("payment #{payment_id} refunded"));
    Ok(())
}

// ── Step 3: Schedule Shipment — always FAILS ──────────────────────────────────

#[ironcmd(result, rename = "ScheduleShipment")]
pub fn schedule_shipment(shipment_id: u64) -> anyhow::Result<u64> {
    println!("[3] Scheduling shipment #{shipment_id}...");
    anyhow::bail!("shipment service unavailable!");
}

// ── Runner ────────────────────────────────────────────────────────────────────

pub fn sync_example() {
    let ctx = Rc::new(RefCell::new(OrderContext::default()));

    let mut create = CreateOrder::new(1001, ctx.clone());
    create.set_rollback(CancelOrder::new(1001, ctx.clone()));

    let mut charge = ChargePayment::new(2002, ctx.clone());
    charge.set_rollback(RefundPayment::new(2002, ctx.clone()));

    let ship = ScheduleShipment::new(3003);

    let mut saga = IronSagaSync::default();
    saga.add_sync_command(create);
    saga.add_sync_command(charge);
    saga.add_sync_command(ship);

    assert!(saga.execute_all().is_err());

    let ctx = ctx.borrow();

    // commands that succeeded still wrote to context
    assert_eq!(ctx.order_id, Some(1001));
    assert_eq!(ctx.payment_id, Some(2002));
    assert_eq!(ctx.shipment_id, None); // never reached

    // rollbacks fired in LIFO order
    assert_eq!(ctx.rollback_log.len(), 2);
    assert_eq!(ctx.rollback_log[0], "payment #2002 refunded");
    assert_eq!(ctx.rollback_log[1], "order #1001 cancelled");
}
 ```
<!-- crate documentation end -->
