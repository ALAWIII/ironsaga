use std::{cell::RefCell, rc::Rc};

use ironsaga::{IronSagaAsync, anyhow, ironcmd};

// ==================== context used to collect results and use them among commands, you can define it as you wish ==============.
#[derive(Default)]
pub struct Context {
    pub full_name: Option<String>,
    pub salary: Option<u32>,
    pub removed_user: bool,
}

// ======================================= you can provide custom names instead of the default ===================

#[ironcmd(rename = "InsertUser", result)]
pub async fn insert_user_db<'a, 'b>(
    fname: &'a str,
    lname: &'b str,
    ctx: Rc<RefCell<Context>>,
) -> Result<String, &'static str> {
    let full_name = format!("{} {}", fname, lname);
    if full_name.len() > 20 {
        return Err("full name exceeded the limit.");
    }
    ctx.borrow_mut().full_name = Some(full_name.to_string());
    Ok(full_name)
}
/// the rollback is also a command but used to be injected within other commands as rollback_cmd field value and executed once the major command fails .
#[ironcmd(recursive_rollback, result)]
pub async fn remove_user_db(ctx: Rc<RefCell<Context>>) -> Result<(), &'static str> {
    ctx.borrow_mut().removed_user = true;
    // some rollback logic
    Ok(())
}
// ==================================== generates AddBonusSalary, in the IronSagaAsyn you can mix between sync and async commands freely=============
#[ironcmd(result)]
pub fn add_bonus_salary<'a>(
    salary: &'a str,
    bonus: u32,
    ctx: Rc<RefCell<Context>>,
) -> anyhow::Result<u32> {
    let salary: u32 = salary.parse()?;
    let new_salary = salary + bonus;
    ctx.borrow_mut().salary = Some(new_salary);
    Ok(new_salary)
}
#[ironcmd(result, recursive_rollback)]
pub async fn add_user_redis(ctx: Rc<RefCell<Context>>) -> Result<String, &'static str> {
    if let Some(s) = ctx.borrow().salary
        && s > 5000
    {
        return Err("you are not allowed to be rich.");
    }
    Ok("horay user added to redis".into())
}
// ====================
/// IronSagaAsync can include both sync and async commands.
pub async fn async_example() {
    let mut bus = IronSagaAsync::default();
    let ctx = Rc::new(RefCell::new(Context::default()));
    assert!(!ctx.borrow().removed_user, "the rollback didnt run yet.");
    let fname = "salah";
    let lname = "aldeen";
    let mut user_insertion = InsertUser::new(fname, lname, ctx.clone());
    let rollback = RemoveUserDb::new(ctx.clone());
    user_insertion.set_rollback_async(rollback); // adding a rollback for the insertion command

    let bonus = AddBonusSalary::new("99", 5000, ctx.clone());
    let rds_user = AddUserRedis::new(ctx.clone());
    // we can assign a rollback for a command , rollbacks are also commands but invoked on the future commands failures!
    // now we add all main commands
    bus.add_async_command(user_insertion);
    bus.add_sync_command(bonus);
    bus.add_async_command(rds_user);
    //-------------------------------------- executing all commands
    assert!(
        bus.execute_all().await.is_err(),
        "command AddUserRedis will fail, so the bus rollsback all previous success executed commands `if they have a rollback to be executed`, then propogates the error."
    );
    //-----------------------
    assert_eq!(ctx.borrow().full_name, Some("salah aldeen".into()));
    assert!(
        ctx.borrow().removed_user,
        "if the rollback works then it must equal to true"
    );
    assert_eq!(ctx.borrow().salary, Some(5099));
}
