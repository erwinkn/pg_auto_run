use pg_query::ast::*;

use crate::objects::*;
use crate::parser::*;

impl SchemaParser {
    pub(crate) fn create_schema(&mut self, stmt: CreateSchemaStmt) {
        // 0+ schema elements
        if let Some(elts) = stmt.schema_elts {
            for elt in elts {
                self.parse_node(elt);
            }
        }

        let mut owner: Option<String> = None;

        if let Some(role) = stmt.authrole {
            owner = role.rolename;
        }
        // Schema creation
        // (ignore IF NOT EXISTS)
        let name = QualifiedName {
            name: stmt.schemaname.unwrap(),
            schema_name: None,
        };
        self.create_object(Object {
            name,
            entity: Entity::Schema(Schema { owner }),
        });
    }

    pub(crate) fn create_table(&mut self, stmt: CreateStmt) {
        // Table creation
        self.create_object(Object {
            name: range_var_name(*stmt.relation.unwrap()),
            entity: Entity::Table,
        });

        // 0~1 TYPE OF
        if let Some(typename) = stmt.of_typename {
            let name = schema_qualified_name(typename.names.unwrap());
            self.register_dep(name, EntityTag::Type);
        }

        // 1+ column definitions
        let elts = stmt.table_elts.unwrap();
        for node in elts {
            match node {
                Node::ColumnDef(def) => self.column_def(def),
                Node::Constraint(constraint) => self.constraint(constraint),
                Node::TableLikeClause(clause) => {
                    let name = range_var_name(*clause.relation.unwrap());
                    self.register_dep(name, EntityTag::Table);
                }
                _ => unreachable!(),
            }
        }

        // 0+ INHERITS
        if let Some(parents) = stmt.inh_relations {
            for node in parents {
                let name = expect_range_var(node);
                self.register_dep(name, EntityTag::Table);
            }
        }

        // 0~1 tablespace
        if let Some(tablespace) = stmt.tablespacename {
            let name = QualifiedName {
                name: tablespace,
                schema_name: None,
            };
            self.register_dep(name, EntityTag::Tablespace);
        }
    }

    fn column_def(&mut self, def: ColumnDef) {
        // Type dependency
        let t = def.type_name.unwrap();
        let name = schema_qualified_name(t.names.unwrap());
        self.register_dep(name, EntityTag::Type);

        // 1+ constraints (optional)
        if let Some(constraints) = def.constraints {
            for node in constraints {
                match node {
                    Node::Constraint(c) => self.constraint(c),
                    _ => unreachable!(),
                }
            }
        }
        // 1 collation (optional)
        if let Some(collation) = def.coll_clause {
            let name = schema_qualified_name(collation.collname.unwrap());
            self.register_dep(name, EntityTag::Collation);
        }
    }

    // TODO: do we need to register constraints
    pub(crate) fn constraint(&mut self, constraint: Constraint) {
        //
    }

    pub(crate) fn define_stmt(&mut self, stmt: DefineStmt) {
        match *stmt.kind {
            ObjectType::OBJECT_COLLATION => {
                let name = schema_qualified_name(stmt.defnames.unwrap());
                self.create_object(Object {
                    name,
                    entity: Entity::Collation,
                });
                // Look for dependencies as a FROM expression
                if let Some(defs) = stmt.definition {
                    for d in defs {
                        match d {
                            Node::DefElem(elem) => self.def_elem(elem, EntityTag::Collation),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            _ => todo!(),
        }
    }

    // Only accessed through a DefineStmt corresponding to a CREATE COLLATION atm
    pub(crate) fn def_elem(&mut self, elem: DefElem, dep_type: EntityTag) {
        match elem.defname {
            Some(n) if n == "from" => {
                let node = elem.arg.unwrap();
                match *node {
                    Node::List(s) => {
                        let name = schema_qualified_name(s.items);
                        self.register_dep(name, dep_type);
                    }
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    }
}
