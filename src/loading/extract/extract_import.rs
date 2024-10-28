use std::collections::HashMap;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_import_section(section: &CafImport, imports: &mut HashMap<ManifestKey, CafImportAlias>)
{
    for entry in section.entries.iter() {
        imports.insert(entry.key.clone(), entry.alias.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------
