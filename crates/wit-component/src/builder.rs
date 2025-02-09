use std::mem;
use wasm_encoder::*;

/// Helper type used when encoding a component to have helpers that
/// simultaneously encode an item while returning its corresponding index in the
/// generated index spaces as well.
#[derive(Default)]
pub struct ComponentBuilder {
    /// The binary component as created by `wasm-encoder`.
    component: Component,

    /// The last section which was appended to during encoding. This type is
    /// generated by the `section_accessors` macro below.
    ///
    /// When something is encoded this is used if it matches the kind of item
    /// being encoded, otherwise it's "flushed" to the output component and a
    /// new section is started.
    last_section: LastSection,

    // Core index spaces
    core_modules: u32,
    core_funcs: u32,
    core_memories: u32,
    core_tables: u32,
    core_instances: u32,

    // Component index spaces
    funcs: u32,
    instances: u32,
    types: u32,
}

impl ComponentBuilder {
    pub fn finish(mut self) -> Vec<u8> {
        self.flush();
        self.component.finish()
    }

    pub fn instantiate<'a, A>(&mut self, module_index: u32, args: A) -> u32
    where
        A: IntoIterator<Item = (&'a str, ModuleArg)>,
        A::IntoIter: ExactSizeIterator,
    {
        self.instances().instantiate(module_index, args);
        inc(&mut self.core_instances)
    }

    pub fn alias_func(&mut self, instance: u32, name: &str) -> u32 {
        self.aliases().alias(Alias::InstanceExport {
            instance,
            kind: ComponentExportKind::Func,
            name,
        });
        inc(&mut self.funcs)
    }

    pub fn lower_func<O>(&mut self, func_index: u32, options: O) -> u32
    where
        O: IntoIterator<Item = CanonicalOption>,
        O::IntoIter: ExactSizeIterator,
    {
        self.canonical_functions().lower(func_index, options);
        inc(&mut self.core_funcs)
    }

    pub fn lift_func<O>(&mut self, core_func_index: u32, type_index: u32, options: O) -> u32
    where
        O: IntoIterator<Item = CanonicalOption>,
        O::IntoIter: ExactSizeIterator,
    {
        self.canonical_functions()
            .lift(core_func_index, type_index, options);
        inc(&mut self.funcs)
    }

    pub fn instantiate_core_exports<'a, E>(&mut self, exports: E) -> u32
    where
        E: IntoIterator<Item = (&'a str, ExportKind, u32)>,
        E::IntoIter: ExactSizeIterator,
    {
        self.instances().export_items(exports);
        inc(&mut self.core_instances)
    }

    pub fn instantiate_exports<'a, E>(&mut self, exports: E) -> u32
    where
        E: IntoIterator<Item = (&'a str, ComponentExportKind, u32)>,
        E::IntoIter: ExactSizeIterator,
    {
        self.component_instances().export_items(exports);
        inc(&mut self.instances)
    }

    pub fn core_module(&mut self, module: &Module) -> u32 {
        self.flush();
        self.component.section(&ModuleSection(module));
        inc(&mut self.core_modules)
    }

    pub fn core_module_raw(&mut self, module: &[u8]) -> u32 {
        self.flush();
        self.component.section(&wasm_encoder::RawSection {
            id: ComponentSectionId::CoreModule.into(),
            data: module,
        });
        inc(&mut self.core_modules)
    }

    pub fn alias_core_item(&mut self, instance: u32, kind: ExportKind, name: &str) -> u32 {
        self.aliases().alias(Alias::CoreInstanceExport {
            instance,
            kind,
            name,
        });
        match kind {
            ExportKind::Func => inc(&mut self.core_funcs),
            ExportKind::Table => inc(&mut self.core_tables),
            ExportKind::Memory => inc(&mut self.core_memories),
            ExportKind::Global | ExportKind::Tag => unreachable!(),
        }
    }

    pub fn export(&mut self, name: &str, url: &str, kind: ComponentExportKind, idx: u32) -> u32 {
        self.exports().export(name, url, kind, idx);
        match kind {
            ComponentExportKind::Type => inc(&mut self.types),
            ComponentExportKind::Func => inc(&mut self.funcs),
            ComponentExportKind::Module => inc(&mut self.core_modules),
            ComponentExportKind::Instance => inc(&mut self.instances),
            ComponentExportKind::Component | ComponentExportKind::Value => unimplemented!(),
        }
    }

    pub fn import(&mut self, name: &str, url: &str, ty: ComponentTypeRef) -> u32 {
        let ret = match &ty {
            ComponentTypeRef::Instance(_) => inc(&mut self.instances),
            ComponentTypeRef::Func(_) => inc(&mut self.funcs),
            _ => unimplemented!(),
        };
        self.imports().import(name, url, ty);
        ret
    }

    pub fn instance_type(&mut self, ty: &InstanceType) -> u32 {
        let ret = inc(&mut self.types);
        self.types().instance(ty);
        ret
    }

    pub fn defined_type(&mut self) -> (u32, ComponentDefinedTypeEncoder<'_>) {
        (inc(&mut self.types), self.types().defined_type())
    }

    pub fn function_type(&mut self) -> (u32, ComponentFuncTypeEncoder<'_>) {
        (inc(&mut self.types), self.types().function())
    }

    pub fn alias_type_export(&mut self, instance: u32, name: &str) -> u32 {
        self.aliases().alias(Alias::InstanceExport {
            instance,
            kind: ComponentExportKind::Type,
            name,
        });
        inc(&mut self.types)
    }

    pub fn alias_outer_type(&mut self, count: u32, index: u32) -> u32 {
        self.aliases().alias(Alias::Outer {
            count,
            kind: ComponentOuterAliasKind::Type,
            index,
        });
        inc(&mut self.types)
    }
}

// Helper macro to generate methods on `ComponentBuilder` to get specific
// section encoders that automatically flush and write out prior sections as
// necessary.
macro_rules! section_accessors {
    ($($method:ident => $section:ident)*) => (
        #[derive(Default)]
        enum LastSection {
            #[default]
            None,
            $($section($section),)*
        }

        impl ComponentBuilder {
            $(
                fn $method(&mut self) -> &mut $section {
                    match &self.last_section {
                        // The last encoded section matches the section that's
                        // being requested, so no change is necessary.
                        LastSection::$section(_) => {}

                        // Otherwise the last section didn't match this section,
                        // so flush any prior section if needed and start
                        // encoding the desired section of this method.
                        _ => {
                            self.flush();
                            self.last_section = LastSection::$section($section::new());
                        }
                    }
                    match &mut self.last_section {
                        LastSection::$section(ret) => ret,
                        _ => unreachable!()
                    }
                }
            )*

            /// Writes out the last section into the final component binary if
            /// there is a section specified, otherwise does nothing.
            fn flush(&mut self) {
                match mem::take(&mut self.last_section) {
                    LastSection::None => {}
                    $(
                        LastSection::$section(section) => {
                            self.component.section(&section);
                        }
                    )*
                }
            }

        }
    )
}

section_accessors! {
    component_instances => ComponentInstanceSection
    instances => InstanceSection
    canonical_functions => CanonicalFunctionSection
    aliases => ComponentAliasSection
    exports => ComponentExportSection
    imports => ComponentImportSection
    types => ComponentTypeSection
}

fn inc(idx: &mut u32) -> u32 {
    let ret = *idx;
    *idx += 1;
    ret
}
