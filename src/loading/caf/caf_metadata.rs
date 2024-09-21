//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafMetadata
{
    /// Location of the caf file within the project's `assets` directory.
    pub file_path: String,
    /// Estimated tab width policy of the file.
    ///
    /// Can be used when inserting new content to calculate the correct position for placement.
    pub tab_width: u8,
}

impl Default for CafMetadata
{
    fn default() -> Self
    {
        Self { file_path: String::default(), tab_width: 4 }
    }
}

//-------------------------------------------------------------------------------------------------------------------
