pub use anyhow;
pub use async_trait;
pub use ironsaga_macros::ironcmd;

#[async_trait::async_trait]
pub trait AsyncCommand: Send {
    async fn execute(&mut self) -> anyhow::Result<()>;
    async fn rollback(&mut self) -> anyhow::Result<()>;
}
pub trait SyncCommand {
    fn execute(&mut self) -> anyhow::Result<()>;
    fn rollback(&mut self) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct IronSagaAsync<'a> {
    commands: Vec<Box<dyn AsyncCommand + 'a>>,
}
impl<'a> IronSagaAsync<'a> {
    pub async fn execute_all(&mut self) -> anyhow::Result<()> {
        for (i, c) in self.commands.iter_mut().enumerate() {
            let r = c.execute().await;
            if let Err(er) = r {
                self.rollback_all(i).await?;
                return Err(er);
            }
        }
        Ok(())
    }
    async fn rollback_all(&mut self, index: usize) -> anyhow::Result<()> {
        for i in (0..index).rev() {
            self.commands.get_mut(i).unwrap().rollback().await?;
        }
        Ok(())
    }
    pub fn add_command(&mut self, c: impl AsyncCommand + 'a) {
        self.commands.push(Box::new(c));
    }
    pub fn commands(&self) -> &[Box<dyn AsyncCommand + 'a>] {
        &self.commands
    }
    pub fn commands_mut(&mut self) -> &mut [Box<dyn AsyncCommand + 'a>] {
        &mut self.commands
    }
}
#[derive(Default)]
pub struct IronSagaSync<'a> {
    commands: Vec<Box<dyn SyncCommand + 'a>>,
}
impl<'a> IronSagaSync<'a> {
    pub fn execute_all(&mut self) -> anyhow::Result<()> {
        for (i, c) in self.commands.iter_mut().enumerate() {
            let r = c.execute();
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
    pub fn add_command(&mut self, c: impl SyncCommand + 'a) {
        self.commands.push(Box::new(c));
    }
    pub fn commands(&self) -> &[Box<dyn SyncCommand + 'a>] {
        &self.commands
    }
    pub fn commands_mut(&mut self) -> &mut [Box<dyn SyncCommand + 'a>] {
        &mut self.commands
    }
}
