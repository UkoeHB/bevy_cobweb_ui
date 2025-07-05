use std::collections::HashMap;

use crate::cob::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_import_section(section: &CobImport, imports: &mut HashMap<ManifestKey, CobImportAlias>)
{
    for entry in section.entries.iter() {
        imports.insert(entry.key.clone(), entry.alias.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------
