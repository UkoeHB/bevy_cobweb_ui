use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Collection of resolvers for `CobLoadables`.
#[derive(Default, Debug)]
pub struct CobLoadableResolver
{
    pub constants: ConstantsResolver,
}

impl CobLoadableResolver
{
    pub(crate) fn start_new_file(&mut self)
    {
        self.constants.start_new_file();
    }

    pub(crate) fn end_new_file(&mut self)
    {
        self.constants.end_new_file();
    }

    pub(crate) fn append(&mut self, alias: &CobImportAlias, to_append: &Self)
    {
        self.constants.append(alias, &to_append.constants);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Collection of resolvers for `CobSceneLayers`.
#[derive(Default, Debug)]
pub struct CobSceneResolver
{
    pub scene_macros: SceneMacrosResolver,
}

impl CobSceneResolver
{
    pub(crate) fn start_new_file(&mut self)
    {
        self.scene_macros.start_new_file();
    }

    pub(crate) fn end_new_file(&mut self)
    {
        self.scene_macros.end_new_file();
    }

    pub(crate) fn append(&mut self, alias: &CobImportAlias, to_append: &Self)
    {
        self.scene_macros.append(alias, &to_append.scene_macros);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Collection of resolvers for `Cob` structures.
#[derive(Default, Debug)]
pub struct CobResolver
{
    pub loadables: CobLoadableResolver,
    pub scenes: CobSceneResolver,
}

impl CobResolver
{
    pub(crate) fn start_new_file(&mut self)
    {
        self.loadables.start_new_file();
        self.scenes.start_new_file();
    }

    pub(crate) fn end_new_file(&mut self)
    {
        self.loadables.end_new_file();
        self.scenes.end_new_file();
    }

    pub(crate) fn append(&mut self, alias: &CobImportAlias, to_append: &Self)
    {
        self.loadables.append(alias, &to_append.loadables);
        self.scenes.append(alias, &to_append.scenes);
    }
}

//-------------------------------------------------------------------------------------------------------------------
