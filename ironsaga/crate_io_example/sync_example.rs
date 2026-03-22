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
