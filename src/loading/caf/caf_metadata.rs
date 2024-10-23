//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafMetadata
{
    /// Location of the caf file within the project's `assets` directory.
    pub file_path: String,
}

impl Default for CafMetadata
{
    fn default() -> Self
    {
        Self { file_path: String::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------
