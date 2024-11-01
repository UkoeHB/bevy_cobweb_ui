

//-------------------------------------------------------------------------------------------------------------------

struct FileCommandsInfo
{
    /// Cached commands
    commands: Vec<bool>,
    /// Files in this file's manifest. Commands in these files will be applied immediately after the commands in
    /// this file.
    dependent_files: Vec<CafFile>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Manages commands loaded from CAF files to ensure they are applied in global order.
#[derive(Default, Debug)]
pub(super) struct CommandsBuffer
{

}

//-------------------------------------------------------------------------------------------------------------------
