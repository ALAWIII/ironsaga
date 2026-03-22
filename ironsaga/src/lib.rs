pub use anyhow;
pub use async_trait;
pub use ironsaga_macros::ironcmd;
#[async_trait::async_trait(?Send)]
pub trait AsyncCommand {
    async fn execute(&mut self) -> anyhow::Result<()>;
    async fn rollback(&mut self) -> anyhow::Result<()>;
}
pub trait SyncCommand {
    fn execute(&mut self) -> anyhow::Result<()>;
    fn rollback(&mut self) -> anyhow::Result<()>;
}
pub enum CommandKind<'a> {
    AsyncCmd(Box<dyn AsyncCommand + 'a>),
    SyncCmd(Box<dyn SyncCommand + 'a>),
}
pub struct IronSagaAsync<'a> {
    commands: Vec<CommandKind<'a>>,
}
impl<'a> IronSagaAsync<'a> {
    pub async fn execute_all(&mut self) -> anyhow::Result<()> {
        for (i, v) in self.commands.iter_mut().enumerate() {
            let r = match v {
                CommandKind::AsyncCmd(c) => c.execute().await,
                CommandKind::SyncCmd(c) => c.execute(),
            };
            if let Err(er) = r {
                self.rollback_all(i).await?;
                return Err(er);
            }
        }
        Ok(())
    }
    async fn rollback_all(&mut self, index: usize) -> anyhow::Result<()> {
        for i in (0..index).rev() {
            match self.commands.get_mut(i).unwrap() {
                CommandKind::AsyncCmd(c) => c.rollback().await,
                CommandKind::SyncCmd(c) => c.rollback(),
            }?;
        }
        Ok(())
    }
    pub fn add_async_command(&mut self, c: impl AsyncCommand + 'a) {
        let ac = CommandKind::AsyncCmd(Box::new(c));
        self.commands.push(ac);
    }
    pub fn add_sync_command(&mut self, c: impl SyncCommand + 'a) {
        let ac = CommandKind::SyncCmd(Box::new(c));
        self.commands.push(ac);
    }
}

pub struct IronSagaSync<'a> {
    commands: Vec<Box<dyn SyncCommand + 'a>>,
}
impl<'a> IronSagaSync<'a> {
    pub fn execute_all(&mut self) -> anyhow::Result<()> {
        for (i, v) in self.commands.iter_mut().enumerate() {
            let r = v.execute();
            if let Err(er) = r {
                self.rollback_all(i)?;
                return Err(er);
            }
        }
        Ok(())
    }
    fn rollback_all(&mut self, index: usize) -> anyhow::Result<()> {
        for i in (0..index).rev() {
            self.commands.get_mut(i).unwrap().rollback()?;
        }
        Ok(())
    }
    pub fn add_sync_command(&mut self, c: impl SyncCommand + 'a) {
        self.commands.push(Box::new(c));
    }
}
