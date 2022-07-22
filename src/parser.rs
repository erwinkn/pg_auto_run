
mod parser {
    use std::{collections::HashMap, fs};
    
    use ignore::DirEntry;
    
    use pg_query::ast::*;
    
    pub(crate) struct SchemaParser {
        files: Vec<DirEntry>,
        objects: Vec<Object>,
        refs: HashMap<QualifiedName, ObjectId>,
        creations: Vec<Vec<ObjectId>>,
        dependencies: Vec<Vec<ObjectId>>,
        // Creations and dependencies for the current file
        x_creations: Option<Vec<ObjectId>>,
        x_dependencies: Option<Vec<ObjectId>>,
    }
    
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct ObjectId(usize);
    
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct FileId(usize);
    
    impl SchemaParser {
        pub fn new() -> Self {
            Self {
                objects: vec![],
                files: vec![],
                refs: HashMap::new(),
                creations: vec![],
                dependencies: vec![],
                x_creations: None,
                x_dependencies: None,
            }
        }
    
        pub fn parse(&mut self, file: DirEntry) {
            let path = file.path();
            let path_str = path.to_str().unwrap().to_string();
            println!("[File: {}]", path_str);
            let contents = fs::read_to_string(path).unwrap();
            let ast = pg_query::parse(&contents).unwrap();
            println!("{:#?}", ast);
    
            let _file_id = self.insert_file(file);
    
            self.x_creations = Some(vec![]);
            self.x_dependencies = Some(vec![]);
    
            for node in ast {
                self.parse_node(node)
            }
            self.creations.push(self.x_creations.take().unwrap());
            self.dependencies.push(self.x_dependencies.take().unwrap());
        }
    
        pub(crate) fn parse_node(&mut self, node: Node) {
    
                match node {
                    Node::CreateSchemaStmt(s) => self.create_schema(s),
                    Node::CreateStmt(s) => self.create_table(s),
                    // Only supports CREATE COLLATION atm
                    Node::DefineStmt(s) => self.define_stmt(s),
                    Node::VariableSetStmt(..) => panic!("Setting session variables is not supported"),
                    // Node::CreateSchemaStmt(s) => create_schema(s),
                    // Node::ViewStmt(s) => create_view(s),
                    // Node::CreateFunctionStmt(s) => create_function(s),
                    // Node::CreateTableAsStmt(s) => create_table_as(s),
                    // Node::CreateEnumStmt(s) => create_enum(s),
                    _ => {}
                }
        }
    
        pub(crate) fn insert_file(&mut self, file: DirEntry) -> FileId {
            self.files.push(file);
            FileId(self.files.len() - 1)
        }
    
        pub(crate) fn create_object(&mut self, object: Object) -> ObjectId {
            match self.refs.get(&object.name) {
                Some(&id) => {
                    let obj = self.get_object(id);
                    self.assert_can_replace(obj, &object);
                    // replace existing object
                    self.objects[id.0] = object;
                    id
                }
                None => {
                    self.objects.push(object);
                    let id = ObjectId(self.objects.len() - 1);
                    self.x_creations.as_mut().unwrap().push(id);
                    id
                }
            }
        }
    
        pub(crate) fn register_dep(&mut self, name: QualifiedName, tag: EntityTag) -> ObjectId {
            match self.refs.get(&name) {
                Some(&id) => id,
                None => {
                    // TODO: can we remove this clone?
                    self.objects.push(Object {
                        name: name.clone(),
                        entity: Entity::Placeholder(tag),
                    });
                    let id = ObjectId(self.objects.len() - 1);
                    self.refs.insert(name, id);
                    id
                }
            }
        }
    
        pub(crate) fn get_file(&self, id: FileId) -> &DirEntry {
            self.files.get(id.0).unwrap()
        }
    
        pub(crate) fn current_file(&self) -> &DirEntry {
            self.files.last().unwrap()
        }
    
        pub(crate) fn get_object(&self, id: ObjectId) -> &Object {
            self.objects.get(id.0).unwrap()
        }
    
    
    
    
        // TODO: improve
        pub(crate) fn assert_can_replace(&self, existing: &Object, new: &Object) {
            debug_assert!(existing.name == new.name);
            let new_tag = EntityTag::from(&new.entity);
            match &existing.entity {
                Entity::Placeholder(tag) => {
                    if *tag != new_tag {
                        // TODO: add pretty printing
                        panic!(
                            "[File: {}] Database object type error: {:?} is known as a {:?}, but referenced as a {:?}", 
                            self.current_file().file_name().to_string_lossy(), 
                            existing.name,
                            tag,
                            new_tag
                        );
                    }
                }
                e => {
                    panic!(
                        "[File: {}] Redefinition of object {:?} with previously known type of {:?} to type {:?}",
                        self.current_file().file_name().to_string_lossy(), 
                        existing.name,
                        EntityTag::from(e),
                        new_tag
                );
                }
            }
        }
    }
    
    // pub(crate) fn create_schema(stmt: CreateSchemaStmt, created: &mut Vec<Entity>) {
    //     let entity = Schema {
    //         name: stmt.schemaname.unwrap(),
    //     };
    //     created.push(Entity::Schema(entity));
    // }
    
    // pub(crate) fn create_view(stmt: ViewStmt, created: &mut Vec<Entity>) {
    //     let view = stmt.view.unwrap();
    //     let entity = View {
    //         name: view.relname.unwrap(),
    //         schema_name: view.schemaname,
    //     };
    //     created.push(Entity::View(entity));
    // }
    // // Handles both CREATE TABLE AS and CREATE MATERIALIZED VIEW
    // pub(crate) fn create_table_as(stmt: CreateTableAsStmt, created: &mut Vec<Entity>) {
    //     let target = stmt.into.unwrap();
    //     let relation_info = target.rel.unwrap();
    
    //     let name = relation_info.relname.unwrap();
    //     let schema_name = relation_info.schemaname;
    
    //     let entity: Entity = match *stmt.relkind {
    //         ObjectType::OBJECT_MATVIEW => {
    //             Entity::MaterializedView(MaterializedView { name, schema_name })
    //         }
    //         ObjectType::OBJECT_TABLE => Entity::Table(Table { name, schema_name }),
    //         _ => unreachable!(),
    //     };
    //     created.push(entity);
    // }
    // pub(crate) fn create_function(stmt: CreateFunctionStmt, created: &mut Vec<Entity>) {
    //     let (schema_name, name) = schema_qualified_name(stmt.funcname.unwrap());
    //     created.push(Entity::Function(Function { name, schema_name }));
    // }
    // pub(crate) fn create_enum(stmt: CreateEnumStmt, created: &mut Vec<Entity>) {
    //     let (schema_name, name) = schema_qualified_name(stmt.type_name.unwrap());
    //     created.push(Entity::Enum(Enum { name, schema_name }));
    // }
    
    pub(crate) fn schema_qualified_name(name: Vec<Node>) -> QualifiedName {
        if name.len() == 1 {
            QualifiedName {
                schema_name: None,
                name: expect_string(name.get(0).unwrap()),
            }
        } else if name.len() == 2 {
            QualifiedName {
                schema_name: Some(expect_string(name.get(0).unwrap())),
                name: expect_string(name.get(1).unwrap()),
            }
        } else {
            panic!(
                "Expected a schema qualified name, received {:#?} instead",
                name
            );
        }
    }
    
    fn expect_string(node: &Node) -> String {
        match node {
            Node::String { value: Some(value) } => value.to_owned(),
            other => panic!("Expected a string name, got {:#?} instead", other),
        }
    }
    
    pub(crate) fn expect_range_var(node: Node) -> QualifiedName {
        match node {
            Node::RangeVar(var) => range_var_name(var),
            other => panic!("Expected a RangeVar, got {:#?} instead", other),
        }
    }
    
    pub(crate) fn range_var_name(var: RangeVar) -> QualifiedName {
        QualifiedName {
            name: var.relname.unwrap(),
            schema_name: var.schemaname,
        }
    }

}

