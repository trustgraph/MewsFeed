use hdk::prelude::*;



mod init;
mod agent;
pub use init::*;
pub use agent::*;


entry_defs![Myself::entry_defs(), AgentStore::entry_defs()]