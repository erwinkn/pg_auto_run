
mod parser {
  use enum_kinds::EnumKind;

  pub(crate) struct Object {
      pub(crate) name: QualifiedName,
      pub(crate) entity: Entity,
  }
  
  #[derive(EnumKind)]
  #[enum_kind(EntityTag)]
  pub(crate) enum Entity {
      // Supported
      Schema(Schema),
      Table,
      ForeignServer,
      ForeignTable,
      Function,
      MaterializedView,
      Tablespace,
      Type,
      View,
      // Unsupported
      Collation,
      // Internal
      Placeholder(EntityTag),
  }
  
  pub(crate) struct Schema {
      pub(crate) owner: Option<String>,
  }
  
  #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
  pub(crate) struct QualifiedName {
      pub(crate) name: String,
      pub(crate) schema_name: Option<String>,
  }
}

